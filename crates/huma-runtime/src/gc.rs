//! Hüma çalışma zamanı için iz sürmeli döngü toplayıcı.
//!
//! Heap iş parçacığı yereldir; bu, ilerideki isolate modelinde her isolate'ın
//! kendi heap'ine sahip olması için bilinçli bir sınırdır. Toplayıcı, heap
//! kenarlarını izleyip dış kökü kalmamış bağlı bileşenlerin kenarlarını
//! kırar. Genç nesneler bir başarılı taramadan sonra eski nesil olarak işaretlenir.

use crate::value::Deger;
use std::cell::{Cell, Ref, RefCell, RefMut};
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::rc::{Rc, Weak};

const COLLECTION_ALLOCATION_INTERVAL: usize = 4_096;
const MINOR_COLLECTIONS_PER_MAJOR: usize = 16;
const MAX_INLINE_TRACE_DEPTH: usize = 128;

trait HeapPayload: 'static {
    fn trace(&self, visitor: &mut dyn FnMut(usize));
    fn clear_edges(&mut self);
}

struct Allocation<T> {
    value: RefCell<T>,
    generation: Cell<u8>,
}

trait ErasedAllocation {
    fn identity(&self) -> usize;
    fn generation(&self) -> u8;
    fn promote(&self);
    fn trace(&self, visitor: &mut dyn FnMut(usize));
    fn clear_edges(&self) -> bool;
}

impl<T: HeapPayload> ErasedAllocation for Allocation<T> {
    fn identity(&self) -> usize {
        self as *const Self as *const () as usize
    }

    fn generation(&self) -> u8 {
        self.generation.get()
    }

    fn promote(&self) {
        self.generation.set(1);
    }

    fn trace(&self, visitor: &mut dyn FnMut(usize)) {
        if let Ok(value) = self.value.try_borrow() {
            value.trace(visitor);
        }
    }

    fn clear_edges(&self) -> bool {
        match self.value.try_borrow_mut() {
            Ok(mut value) => {
                value.clear_edges();
                true
            }
            Err(_) => false,
        }
    }
}

#[derive(Default)]
struct Registry {
    allocations: Vec<Weak<dyn ErasedAllocation>>,
    allocations_since_collection: usize,
    minor_collections_since_major: usize,
    remembered_old: HashSet<usize>,
}

thread_local! {
    static REGISTRY: RefCell<Registry> = RefCell::new(Registry::default());
}

/// Heap taramasının gözlemlenebilir sonucu.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CollectionStats {
    pub kind: CollectionKind,
    pub examined: usize,
    pub reachable: usize,
    pub reclaimed_cycles: usize,
    pub promoted: usize,
    pub young: usize,
    pub old: usize,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum CollectionKind {
    Minor,
    #[default]
    Major,
}

/// Bir yorumlayıcı/VM ömrünün sonunda kalan döngüleri tarayan koruma.
/// Alan en son bırakılmalıdır; böylece diğer kökler önce düşer.
#[doc(hidden)]
#[derive(Debug, Default)]
pub struct HeapSweepGuard;

impl Drop for HeapSweepGuard {
    fn drop(&mut self) {
        collect_cycles();
    }
}

/// Paylaşımlı Hüma heap hücresi.
///
/// `Rc<RefCell<T>>` ile aynı tek-iş-parçacıklı sahiplik semantiğini sunar,
/// ancak bütün hücreler izleme kaydına girer ve döngüler toplanabilir.
pub struct Gc<T> {
    inner: Rc<Allocation<T>>,
}

impl<T> Clone for Gc<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Rc::clone(&self.inner),
        }
    }
}

#[allow(private_bounds)]
impl<T: HeapPayload> Gc<T> {
    pub fn new(value: T) -> Self {
        let inner = Rc::new(Allocation {
            value: RefCell::new(value),
            generation: Cell::new(0),
        });
        let erased: Rc<dyn ErasedAllocation> = inner.clone();
        let weak = Rc::downgrade(&erased);
        drop(erased);

        let should_collect = REGISTRY.with(|registry| {
            let mut registry = registry.borrow_mut();
            registry.allocations.push(weak);
            registry.allocations_since_collection += 1;
            registry.allocations_since_collection >= COLLECTION_ALLOCATION_INTERVAL
        });
        let result = Self { inner };
        if should_collect {
            collect_automatic();
        }
        result
    }

    /// Geçiş yardımcısı: mevcut `RefCell::new` kurucularını heap kaydına alır.
    pub fn from_cell(value: RefCell<T>) -> Self {
        Self::new(value.into_inner())
    }
}

impl<T> Gc<T> {
    pub fn borrow(&self) -> Ref<'_, T> {
        self.inner.value.borrow()
    }

    pub fn borrow_mut(&self) -> RefMut<'_, T> {
        self.remember_if_old();
        self.inner.value.borrow_mut()
    }

    pub fn try_borrow(&self) -> Result<Ref<'_, T>, std::cell::BorrowError> {
        self.inner.value.try_borrow()
    }

    pub fn try_borrow_mut(&self) -> Result<RefMut<'_, T>, std::cell::BorrowMutError> {
        let borrowed = self.inner.value.try_borrow_mut();
        if borrowed.is_ok() {
            self.remember_if_old();
        }
        borrowed
    }

    pub fn as_ptr(this: &Self) -> *const () {
        Rc::as_ptr(&this.inner) as *const ()
    }

    pub fn ptr_eq(left: &Self, right: &Self) -> bool {
        Rc::ptr_eq(&left.inner, &right.inner)
    }

    fn remember_if_old(&self) {
        if self.inner.generation.get() == 1 {
            let identity = Self::as_ptr(self) as usize;
            REGISTRY.with(|registry| {
                registry.borrow_mut().remembered_old.insert(identity);
            });
        }
    }
}

impl<T: fmt::Debug> fmt::Debug for Gc<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.value.fmt(formatter)
    }
}

impl<T: PartialEq> PartialEq for Gc<T> {
    fn eq(&self, other: &Self) -> bool {
        Self::ptr_eq(self, other) || self.inner.value == other.inner.value
    }
}

fn trace_value(value: &Deger, visitor: &mut dyn FnMut(usize), depth: usize) {
    if depth > MAX_INLINE_TRACE_DEPTH {
        return;
    }
    match value {
        Deger::Liste(items) => visitor(Gc::as_ptr(items) as usize),
        Deger::Nesne { alanlar, .. } | Deger::Sozluk(alanlar) => {
            visitor(Gc::as_ptr(alanlar) as usize)
        }
        Deger::Vektor(items) => visitor(Gc::as_ptr(items) as usize),
        Deger::Matris { veri, .. } => visitor(Gc::as_ptr(veri) as usize),
        Deger::Fonksiyon {
            yakalanan_kapsamlar,
            ..
        } => {
            for scope in yakalanan_kapsamlar {
                for child in scope.values() {
                    trace_value(child, visitor, depth + 1);
                }
            }
        }
        Deger::BytecodeFonksiyon {
            yakalanan_degiskenler,
            ..
        } => {
            for child in yakalanan_degiskenler.values() {
                trace_value(child, visitor, depth + 1);
            }
        }
        Deger::Sayi(_)
        | Deger::Metin(_)
        | Deger::Bayt(_)
        | Deger::GorevId(_)
        | Deger::Bos
        | Deger::DahiliFonksiyon(_)
        | Deger::BaglamliDahiliFonksiyon(_)
        | Deger::Sinif { .. }
        | Deger::Hata(_)
        | Deger::Harici(_) => {}
    }
}

impl HeapPayload for Vec<Deger> {
    fn trace(&self, visitor: &mut dyn FnMut(usize)) {
        for value in self {
            trace_value(value, visitor, 0);
        }
    }

    fn clear_edges(&mut self) {
        self.clear();
    }
}

impl HeapPayload for HashMap<String, Deger> {
    fn trace(&self, visitor: &mut dyn FnMut(usize)) {
        for value in self.values() {
            trace_value(value, visitor, 0);
        }
    }

    fn clear_edges(&mut self) {
        self.clear();
    }
}

impl HeapPayload for Vec<f64> {
    fn trace(&self, _visitor: &mut dyn FnMut(usize)) {}

    fn clear_edges(&mut self) {
        self.clear();
    }
}

fn live_allocations() -> Vec<Rc<dyn ErasedAllocation>> {
    REGISTRY.with(|registry| {
        let mut registry = registry.borrow_mut();
        registry
            .allocations
            .retain(|entry| entry.strong_count() > 0);
        registry
            .allocations
            .iter()
            .filter_map(Weak::upgrade)
            .collect::<Vec<_>>()
    })
}

fn collect_automatic() -> CollectionStats {
    let run_major = REGISTRY.with(|registry| {
        registry.borrow().minor_collections_since_major + 1 >= MINOR_COLLECTIONS_PER_MAJOR
    });
    if run_major {
        collect_cycles()
    } else {
        collect_young()
    }
}

/// Genç nesli, yalnız genç nesneleri ve yazma bariyerinin kaydettiği eski
/// nesneleri izleyerek toplar.
pub fn collect_young() -> CollectionStats {
    let (remembered, allocations) = REGISTRY.with(|registry| {
        let mut registry = registry.borrow_mut();
        registry.allocations_since_collection = 0;
        registry.minor_collections_since_major += 1;
        let remembered = std::mem::take(&mut registry.remembered_old);
        drop(registry);
        (remembered, live_allocations())
    });

    let identities = allocations
        .iter()
        .enumerate()
        .map(|(index, allocation)| (allocation.identity(), index))
        .collect::<HashMap<_, _>>();
    let young = allocations
        .iter()
        .enumerate()
        .filter_map(|(index, allocation)| (allocation.generation() == 0).then_some(index))
        .collect::<HashSet<_>>();
    let mut stats = CollectionStats {
        kind: CollectionKind::Minor,
        examined: young.len(),
        old: allocations.len().saturating_sub(young.len()),
        ..CollectionStats::default()
    };
    if young.is_empty() {
        return stats;
    }

    let mut incoming = vec![0_usize; allocations.len()];
    for index in &young {
        allocations[*index].trace(&mut |identity| {
            if let Some(child) = identities
                .get(&identity)
                .filter(|child| young.contains(child))
            {
                incoming[*child] = incoming[*child].saturating_add(1);
            }
        });
    }

    let mut reachable = HashSet::new();
    let mut pending = Vec::new();
    for index in &young {
        let external = Rc::strong_count(&allocations[*index]).saturating_sub(1 + incoming[*index]);
        if external > 0 && reachable.insert(*index) {
            pending.push(*index);
        }
    }
    for identity in remembered {
        let Some(index) = identities.get(&identity) else {
            continue;
        };
        allocations[*index].trace(&mut |child_identity| {
            if let Some(child) = identities
                .get(&child_identity)
                .filter(|child| young.contains(child))
            {
                incoming[*child] = incoming[*child].saturating_add(1);
                if reachable.insert(*child) {
                    pending.push(*child);
                }
            }
        });
    }

    while let Some(index) = pending.pop() {
        allocations[index].trace(&mut |identity| {
            if let Some(child) = identities
                .get(&identity)
                .filter(|child| young.contains(child))
            {
                if reachable.insert(*child) {
                    pending.push(*child);
                }
            }
        });
    }

    for index in young {
        let allocation = &allocations[index];
        if reachable.contains(&index) {
            stats.reachable += 1;
            allocation.promote();
            stats.promoted += 1;
            stats.old += 1;
        } else if allocation.clear_edges() {
            stats.reclaimed_cycles += 1;
            stats.young += 1;
        }
    }
    stats
}

/// Bütün nesilleri tarar; eski nesildeki erişilemez döngüleri de kırar.
pub fn collect_cycles() -> CollectionStats {
    REGISTRY.with(|registry| {
        let mut registry = registry.borrow_mut();
        registry.allocations_since_collection = 0;
        registry.minor_collections_since_major = 0;
        registry.remembered_old.clear();
    });
    let allocations = live_allocations();

    let mut stats = CollectionStats {
        kind: CollectionKind::Major,
        examined: allocations.len(),
        ..CollectionStats::default()
    };
    if allocations.is_empty() {
        return stats;
    }

    let identities = allocations
        .iter()
        .enumerate()
        .map(|(index, allocation)| (allocation.identity(), index))
        .collect::<HashMap<_, _>>();
    let mut incoming = vec![0_usize; allocations.len()];
    for allocation in &allocations {
        allocation.trace(&mut |identity| {
            if let Some(index) = identities.get(&identity) {
                incoming[*index] = incoming[*index].saturating_add(1);
            }
        });
    }

    let mut reachable = HashSet::new();
    let mut pending = Vec::new();
    for (index, allocation) in allocations.iter().enumerate() {
        // `allocations` dizisinin geçici Rc'si bir güçlü başvurudur. Heap
        // içinden gelen başvuruları ve bu geçici başvuruyu çıkardığımızda
        // kalan her başvuru bir dış köktür.
        let external = Rc::strong_count(allocation).saturating_sub(1 + incoming[index]);
        if external > 0 && reachable.insert(index) {
            pending.push(index);
        }
    }

    while let Some(index) = pending.pop() {
        allocations[index].trace(&mut |identity| {
            if let Some(child) = identities.get(&identity) {
                if reachable.insert(*child) {
                    pending.push(*child);
                }
            }
        });
    }

    for (index, allocation) in allocations.iter().enumerate() {
        if reachable.contains(&index) {
            stats.reachable += 1;
            if allocation.generation() == 0 {
                allocation.promote();
                stats.promoted += 1;
            }
        } else if allocation.clear_edges() {
            stats.reclaimed_cycles += 1;
        }
    }

    for allocation in &allocations {
        if allocation.generation() == 0 {
            stats.young += 1;
        } else {
            stats.old += 1;
        }
    }
    stats
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sahipsiz_liste_dongusunu_toplar() {
        let list = Gc::new(Vec::new());
        list.borrow_mut().push(Deger::Liste(list.clone()));
        drop(list);

        let stats = collect_cycles();
        assert_eq!(stats.examined, 1);
        assert_eq!(stats.reclaimed_cycles, 1);
    }

    #[test]
    fn dis_koku_olan_donguyu_korur() {
        let list = Gc::new(Vec::new());
        list.borrow_mut().push(Deger::Liste(list.clone()));

        let stats = collect_cycles();
        assert_eq!(stats.reachable, 1);
        assert_eq!(list.borrow().len(), 1);
    }

    #[test]
    fn minor_yalniz_genc_nesli_tarar() {
        let old = Gc::new(Vec::<Deger>::new());
        let promoted = collect_cycles();
        assert_eq!(promoted.promoted, 1);

        let young = Gc::new(Vec::new());
        young.borrow_mut().push(Deger::Liste(young.clone()));
        drop(young);
        let stats = collect_young();
        assert_eq!(stats.kind, CollectionKind::Minor);
        assert_eq!(stats.examined, 1);
        assert_eq!(stats.old, 1);
        assert_eq!(stats.reclaimed_cycles, 1);
        drop(old);
    }

    #[test]
    fn yazma_bariyeri_eski_nesneden_gence_kenari_korur() {
        let old = Gc::new(Vec::<Deger>::new());
        collect_cycles();
        let young = Gc::new(Vec::<Deger>::new());
        old.borrow_mut().push(Deger::Liste(young.clone()));
        drop(young);

        let stats = collect_young();
        assert_eq!(stats.reachable, 1);
        assert_eq!(stats.promoted, 1);
        assert_eq!(old.borrow().len(), 1);
    }
}
