use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex};

const EN_FAZLA_TENSOR_ELEMANI: usize = 10_000_000;
const EN_FAZLA_GRAF_DUGUMU: usize = 100_000;
const EN_FAZLA_GRAF_ELEMANI: usize = 20_000_000;
const EN_FAZLA_SAYISAL_ISLEM: usize = 100_000_000;

#[derive(Debug, Clone, PartialEq)]
pub enum OpType {
    Leaf,
    Add(u64, u64),
    MatMul {
        left: u64,
        right: u64,
        r1: usize,
        c1: usize,
        c2: usize,
    },
    ReLU(u64),
}

#[derive(Debug, Clone)]
pub struct TensorData {
    pub id: u64,
    pub satirlar: usize,
    pub sutunlar: usize,
    pub veri: Arc<Mutex<Vec<f64>>>,
    pub gradyan: Arc<Mutex<Vec<f64>>>,
    pub requires_grad: bool,
    pub op: OpType,
}

impl PartialEq for TensorData {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl fmt::Display for TensorData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "tensor[{}×{}, id={}]",
            self.satirlar, self.sutunlar, self.id
        )
    }
}

pub struct AutogradGraph {
    next_id: u64,
    total_elements: usize,
    pub nodes: HashMap<u64, TensorData>,
}

impl Default for AutogradGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl AutogradGraph {
    pub fn new() -> Self {
        Self {
            next_id: 1,
            total_elements: 0,
            nodes: HashMap::new(),
        }
    }

    pub fn tensor_olustur(
        &mut self,
        satirlar: usize,
        sutunlar: usize,
        veri: Vec<f64>,
        requires_grad: bool,
    ) -> Result<TensorData, String> {
        let size = Self::boyut_dogrula(satirlar, sutunlar, "tensor_olustur")?;
        if veri.len() != size {
            return Err(format!(
                "tensor_olustur: {}x{} boyut için {} eleman gerekir; {} geldi",
                satirlar,
                sutunlar,
                size,
                veri.len()
            ));
        }
        Self::sonlu_dogrula(&veri, "tensor_olustur")?;
        let id = self.yeni_id(size)?;
        let t = TensorData {
            id,
            satirlar,
            sutunlar,
            veri: Arc::new(Mutex::new(veri)),
            gradyan: Arc::new(Mutex::new(vec![0.0; size])),
            requires_grad,
            op: OpType::Leaf,
        };

        self.nodes.insert(id, t.clone());
        Ok(t)
    }

    pub fn topla(&mut self, t1: &TensorData, t2: &TensorData) -> Result<TensorData, String> {
        self.graf_tensorkimligi_dogrula(t1, "tensor_topla")?;
        self.graf_tensorkimligi_dogrula(t2, "tensor_topla")?;
        if t1.satirlar != t2.satirlar || t1.sutunlar != t2.sutunlar {
            return Err(format!(
                "tensor_topla: boyutlar eşit olmalı; {}x{} ve {}x{} geldi",
                t1.satirlar, t1.sutunlar, t2.satirlar, t2.sutunlar
            ));
        }
        let size = Self::boyut_dogrula(t1.satirlar, t1.sutunlar, "tensor_topla")?;
        let v1 = Self::veri_kopyala(t1, size, "tensor_topla")?;
        let v2 = Self::veri_kopyala(t2, size, "tensor_topla")?;
        let res = v1
            .iter()
            .zip(v2.iter())
            .enumerate()
            .map(|(index, (left, right))| {
                let sonuc = left + right;
                if sonuc.is_finite() {
                    Ok(sonuc)
                } else {
                    Err(format!(
                        "tensor_topla: {index}. elemanda sonlu olmayan sonuç oluştu"
                    ))
                }
            })
            .collect::<Result<Vec<_>, _>>()?;

        let id = self.yeni_id(size)?;
        let req_grad = t1.requires_grad || t2.requires_grad;
        let t = TensorData {
            id,
            satirlar: t1.satirlar,
            sutunlar: t1.sutunlar,
            veri: Arc::new(Mutex::new(res)),
            gradyan: Arc::new(Mutex::new(vec![0.0; size])),
            requires_grad: req_grad,
            op: OpType::Add(t1.id, t2.id),
        };

        self.nodes.insert(id, t.clone());
        Ok(t)
    }

    pub fn matris_carp(&mut self, t1: &TensorData, t2: &TensorData) -> Result<TensorData, String> {
        self.graf_tensorkimligi_dogrula(t1, "tensor_matris_carp")?;
        self.graf_tensorkimligi_dogrula(t2, "tensor_matris_carp")?;
        if t1.sutunlar != t2.satirlar {
            return Err(format!(
                "tensor_matris_carp: boyut uyumsuzluğu: {}x{} * {}x{}",
                t1.satirlar, t1.sutunlar, t2.satirlar, t2.sutunlar
            ));
        }

        let r1 = t1.satirlar;
        let c1 = t1.sutunlar;
        let c2 = t2.sutunlar;
        let left_size = Self::boyut_dogrula(r1, c1, "tensor_matris_carp")?;
        let right_size = Self::boyut_dogrula(t2.satirlar, c2, "tensor_matris_carp")?;
        let result_size = Self::boyut_dogrula(r1, c2, "tensor_matris_carp")?;
        Self::is_yuku_dogrula(r1, c1, c2, "tensor_matris_carp")?;
        let v1 = Self::veri_kopyala(t1, left_size, "tensor_matris_carp")?;
        let v2 = Self::veri_kopyala(t2, right_size, "tensor_matris_carp")?;

        let mut res = vec![0.0; result_size];
        for i in 0..r1 {
            for k in 0..c1 {
                let a = v1[i * c1 + k];
                for j in 0..c2 {
                    let index = i * c2 + j;
                    res[index] = a.mul_add(v2[k * c2 + j], res[index]);
                    if !res[index].is_finite() {
                        return Err(format!(
                            "tensor_matris_carp: ({i}, {j}) elemanında sonlu olmayan sonuç oluştu"
                        ));
                    }
                }
            }
        }

        let id = self.yeni_id(result_size)?;
        let req_grad = t1.requires_grad || t2.requires_grad;
        let t = TensorData {
            id,
            satirlar: r1,
            sutunlar: c2,
            veri: Arc::new(Mutex::new(res)),
            gradyan: Arc::new(Mutex::new(vec![0.0; result_size])),
            requires_grad: req_grad,
            op: OpType::MatMul {
                left: t1.id,
                right: t2.id,
                r1,
                c1,
                c2,
            },
        };

        self.nodes.insert(id, t.clone());
        Ok(t)
    }

    pub fn relu(&mut self, t1: &TensorData) -> Result<TensorData, String> {
        self.graf_tensorkimligi_dogrula(t1, "tensor_relu")?;
        let size = Self::boyut_dogrula(t1.satirlar, t1.sutunlar, "tensor_relu")?;
        let v1 = Self::veri_kopyala(t1, size, "tensor_relu")?;
        let res = v1
            .into_iter()
            .map(|value| if value > 0.0 { value } else { 0.0 })
            .collect();

        let id = self.yeni_id(size)?;
        let t = TensorData {
            id,
            satirlar: t1.satirlar,
            sutunlar: t1.sutunlar,
            veri: Arc::new(Mutex::new(res)),
            gradyan: Arc::new(Mutex::new(vec![0.0; size])),
            requires_grad: t1.requires_grad,
            op: OpType::ReLU(t1.id),
        };

        self.nodes.insert(id, t.clone());
        Ok(t)
    }

    pub fn backward(&mut self, target_id: u64) -> Result<(), String> {
        let target = match self.nodes.get(&target_id) {
            Some(n) => n.clone(),
            None => return Err(format!("Hedef tensor bulunamadı: {}", target_id)),
        };
        if !target.requires_grad {
            return Err(
                "tensor_geri_yayilim: hedef tensor gradyan gerektirmiyor (requires_grad=0)"
                    .to_string(),
            );
        }
        let target_size =
            Self::boyut_dogrula(target.satirlar, target.sutunlar, "tensor_geri_yayilim")?;
        let mut gradients = HashMap::new();
        gradients.insert(target_id, vec![1.0; target_size]);
        let mut gradient_elements = target_size;

        let mut order = self.topolojik_sirala(target_id)?;
        order.reverse();

        for node_id in order {
            let node = self
                .nodes
                .get(&node_id)
                .cloned()
                .ok_or_else(|| format!("tensor_geri_yayilim: tensor bulunamadı: {node_id}"))?;
            let node_size =
                Self::boyut_dogrula(node.satirlar, node.sutunlar, "tensor_geri_yayilim")?;
            let Some(node_grad) = gradients.get(&node_id).cloned() else {
                continue;
            };
            if node_grad.len() != node_size {
                return Err(format!(
                    "tensor_geri_yayilim: {} düğümünün hesaplanan gradyan boyutu bozuk",
                    node_id
                ));
            }

            match node.op.clone() {
                OpType::Leaf => {}
                OpType::Add(left_id, right_id) => {
                    let left = self.tensor_bul(left_id, "tensor_geri_yayilim")?;
                    let right = self.tensor_bul(right_id, "tensor_geri_yayilim")?;
                    if left.requires_grad {
                        Self::hesaplanan_gradyan_ekle(
                            &mut gradients,
                            &mut gradient_elements,
                            left_id,
                            &node_grad,
                            "tensor_geri_yayilim",
                        )?;
                    }
                    if right.requires_grad {
                        Self::hesaplanan_gradyan_ekle(
                            &mut gradients,
                            &mut gradient_elements,
                            right_id,
                            &node_grad,
                            "tensor_geri_yayilim",
                        )?;
                    }
                }
                OpType::MatMul {
                    left: left_id,
                    right: right_id,
                    r1,
                    c1,
                    c2,
                } => {
                    let left = self.tensor_bul(left_id, "tensor_geri_yayilim")?;
                    let right = self.tensor_bul(right_id, "tensor_geri_yayilim")?;
                    let left_size = Self::boyut_dogrula(r1, c1, "tensor_geri_yayilim")?;
                    let right_size =
                        Self::boyut_dogrula(right.satirlar, c2, "tensor_geri_yayilim")?;
                    let output_size = Self::boyut_dogrula(r1, c2, "tensor_geri_yayilim")?;
                    Self::is_yuku_dogrula(r1, c1, c2, "tensor_geri_yayilim")?;
                    if node_grad.len() != output_size
                        || left.satirlar != r1
                        || left.sutunlar != c1
                        || right.satirlar != c1
                        || right.sutunlar != c2
                    {
                        return Err("tensor_geri_yayilim: matris çarpımı grafik boyutları bozuk"
                            .to_string());
                    }
                    let v1 = Self::veri_kopyala(&left, left_size, "tensor_geri_yayilim")?;
                    let v2 = Self::veri_kopyala(&right, right_size, "tensor_geri_yayilim")?;
                    let mut left_delta = vec![0.0; left_size];
                    let mut right_delta = vec![0.0; right_size];

                    for i in 0..r1 {
                        for k in 0..c1 {
                            for j in 0..c2 {
                                left_delta[i * c1 + k] = node_grad[i * c2 + j]
                                    .mul_add(v2[k * c2 + j], left_delta[i * c1 + k]);
                                right_delta[k * c2 + j] = v1[i * c1 + k]
                                    .mul_add(node_grad[i * c2 + j], right_delta[k * c2 + j]);
                            }
                        }
                    }
                    Self::sonlu_dogrula(&left_delta, "tensor_geri_yayilim")?;
                    Self::sonlu_dogrula(&right_delta, "tensor_geri_yayilim")?;
                    if left.requires_grad {
                        Self::hesaplanan_gradyan_ekle(
                            &mut gradients,
                            &mut gradient_elements,
                            left_id,
                            &left_delta,
                            "tensor_geri_yayilim",
                        )?;
                    }
                    if right.requires_grad {
                        Self::hesaplanan_gradyan_ekle(
                            &mut gradients,
                            &mut gradient_elements,
                            right_id,
                            &right_delta,
                            "tensor_geri_yayilim",
                        )?;
                    }
                }
                OpType::ReLU(in_id) => {
                    let input = self.tensor_bul(in_id, "tensor_geri_yayilim")?;
                    if !input.requires_grad {
                        continue;
                    }
                    let input_size =
                        Self::boyut_dogrula(input.satirlar, input.sutunlar, "tensor_geri_yayilim")?;
                    if input_size != node_grad.len() {
                        return Err("tensor_geri_yayilim: ReLU grafik boyutları bozuk".to_string());
                    }
                    let input_data = Self::veri_kopyala(&input, input_size, "tensor_geri_yayilim")?;
                    let delta = input_data
                        .iter()
                        .zip(node_grad.iter())
                        .map(|(value, gradient)| if *value > 0.0 { *gradient } else { 0.0 })
                        .collect::<Vec<_>>();
                    Self::hesaplanan_gradyan_ekle(
                        &mut gradients,
                        &mut gradient_elements,
                        in_id,
                        &delta,
                        "tensor_geri_yayilim",
                    )?;
                }
            }
        }

        // Tüm hesaplamalar ve kilit edinimleri başarıyla tamamlanmadan tek bir
        // gradyan bile değiştirilmez.
        let mut gradient_ids = gradients.keys().copied().collect::<Vec<_>>();
        gradient_ids.sort_unstable();
        let tensors = gradient_ids
            .iter()
            .map(|id| self.tensor_bul(*id, "tensor_geri_yayilim"))
            .collect::<Result<Vec<_>, _>>()?;
        let mut guards = Vec::with_capacity(tensors.len());
        for tensor in &tensors {
            guards.push(
                tensor
                    .gradyan
                    .lock()
                    .map_err(|_| "tensor_geri_yayilim: gradyan kilidi bozuldu".to_string())?,
            );
        }
        for ((id, tensor), guard) in gradient_ids.iter().zip(tensors.iter()).zip(guards.iter()) {
            let expected =
                Self::boyut_dogrula(tensor.satirlar, tensor.sutunlar, "tensor_geri_yayilim")?;
            let computed = gradients
                .get(id)
                .ok_or_else(|| "tensor_geri_yayilim: hesaplanan gradyan kayboldu".to_string())?;
            if guard.len() != expected || computed.len() != expected {
                return Err(format!(
                    "tensor_geri_yayilim: {} gradyanının boyutu bozuk",
                    id
                ));
            }
            Self::sonlu_dogrula(computed, "tensor_geri_yayilim")?;
        }
        for (id, guard) in gradient_ids.iter().zip(guards.iter_mut()) {
            let computed = gradients
                .get(id)
                .ok_or_else(|| "tensor_geri_yayilim: hesaplanan gradyan kayboldu".to_string())?;
            guard.copy_from_slice(computed);
        }
        Ok(())
    }

    fn topolojik_sirala(&self, root_id: u64) -> Result<Vec<u64>, String> {
        let mut states = HashMap::<u64, u8>::new();
        let mut order = Vec::new();
        let mut stack = vec![(root_id, false)];
        while let Some((node_id, expanded)) = stack.pop() {
            if expanded {
                states.insert(node_id, 2);
                order.push(node_id);
                continue;
            }
            match states.get(&node_id).copied() {
                Some(2) => continue,
                Some(1) => {
                    return Err(format!(
                        "tensor_geri_yayilim: döngüsel hesaplama grafiği algılandı: {node_id}"
                    ))
                }
                _ => {}
            }
            let node = self.nodes.get(&node_id).ok_or_else(|| {
                format!("tensor_geri_yayilim: grafik düğümü bulunamadı: {node_id}")
            })?;
            states.insert(node_id, 1);
            stack.push((node_id, true));
            match &node.op {
                OpType::Leaf => {}
                OpType::Add(left, right) | OpType::MatMul { left, right, .. } => {
                    stack.push((*right, false));
                    stack.push((*left, false));
                }
                OpType::ReLU(input) => stack.push((*input, false)),
            }
            if stack.len() > EN_FAZLA_GRAF_DUGUMU.saturating_mul(3) {
                return Err(
                    "tensor_geri_yayilim: topolojik çalışma yığını sınırı aşıldı".to_string(),
                );
            }
        }
        Ok(order)
    }

    fn yeni_id(&mut self, element_count: usize) -> Result<u64, String> {
        if self.nodes.len() >= EN_FAZLA_GRAF_DUGUMU {
            return Err(format!(
                "tensor: grafik düğüm sınırı aşıldı ({EN_FAZLA_GRAF_DUGUMU})"
            ));
        }
        let next_total = self
            .total_elements
            .checked_add(element_count)
            .ok_or_else(|| "tensor: grafik eleman sayısı taştı".to_string())?;
        if next_total > EN_FAZLA_GRAF_ELEMANI {
            return Err(format!(
                "tensor: toplam grafik eleman sınırı aşıldı ({EN_FAZLA_GRAF_ELEMANI})"
            ));
        }
        let id = self.next_id;
        self.next_id = self
            .next_id
            .checked_add(1)
            .ok_or_else(|| "tensor: kimlik alanı tükendi".to_string())?;
        self.total_elements = next_total;
        Ok(id)
    }

    fn boyut_dogrula(satirlar: usize, sutunlar: usize, islem: &str) -> Result<usize, String> {
        if satirlar == 0 || sutunlar == 0 {
            return Err(format!("{islem}: tensor boyutları pozitif olmalı"));
        }
        let size = satirlar
            .checked_mul(sutunlar)
            .ok_or_else(|| format!("{islem}: tensor boyut çarpımı taştı"))?;
        if size > EN_FAZLA_TENSOR_ELEMANI {
            return Err(format!(
                "{islem}: {size} eleman güvenlik sınırını ({EN_FAZLA_TENSOR_ELEMANI}) aşıyor"
            ));
        }
        Ok(size)
    }

    fn is_yuku_dogrula(
        satirlar: usize,
        ortak: usize,
        sutunlar: usize,
        islem: &str,
    ) -> Result<(), String> {
        let work = satirlar
            .checked_mul(ortak)
            .and_then(|work| work.checked_mul(sutunlar))
            .ok_or_else(|| format!("{islem}: sayısal iş yükü hesabı taştı"))?;
        if work > EN_FAZLA_SAYISAL_ISLEM {
            return Err(format!(
                "{islem}: {work} işlem güvenlik sınırını ({EN_FAZLA_SAYISAL_ISLEM}) aşıyor"
            ));
        }
        Ok(())
    }

    fn hesaplanan_gradyan_ekle(
        gradients: &mut HashMap<u64, Vec<f64>>,
        total_elements: &mut usize,
        tensor_id: u64,
        delta: &[f64],
        islem: &str,
    ) -> Result<(), String> {
        Self::sonlu_dogrula(delta, islem)?;
        if let Some(current) = gradients.get_mut(&tensor_id) {
            if current.len() != delta.len() {
                return Err(format!(
                    "{islem}: hesaplanan gradyan boyutları uyuşmuyor; {} ve {}",
                    current.len(),
                    delta.len()
                ));
            }
            let mut next = Vec::with_capacity(current.len());
            for (index, (left, right)) in current.iter().zip(delta.iter()).enumerate() {
                let value = left + right;
                if !value.is_finite() {
                    return Err(format!(
                        "{islem}: {index}. gradyan elemanında sonlu olmayan sonuç oluştu"
                    ));
                }
                next.push(value);
            }
            *current = next;
            return Ok(());
        }
        let next_total = total_elements
            .checked_add(delta.len())
            .ok_or_else(|| format!("{islem}: gradyan çalışma belleği hesabı taştı"))?;
        if next_total > EN_FAZLA_GRAF_ELEMANI {
            return Err(format!(
                "{islem}: gradyan çalışma belleği {} eleman sınırını aşıyor",
                EN_FAZLA_GRAF_ELEMANI
            ));
        }
        *total_elements = next_total;
        gradients.insert(tensor_id, delta.to_vec());
        Ok(())
    }

    fn sonlu_dogrula(values: &[f64], islem: &str) -> Result<(), String> {
        if let Some((index, _)) = values
            .iter()
            .enumerate()
            .find(|(_, value)| !value.is_finite())
        {
            return Err(format!("{islem}: {index}. eleman sonlu sayı olmalı"));
        }
        Ok(())
    }

    fn graf_tensorkimligi_dogrula(&self, tensor: &TensorData, islem: &str) -> Result<(), String> {
        let stored = self
            .nodes
            .get(&tensor.id)
            .ok_or_else(|| format!("{islem}: tensor grafikte bulunamadı: {}", tensor.id))?;
        if stored.satirlar != tensor.satirlar || stored.sutunlar != tensor.sutunlar {
            return Err(format!("{islem}: tensor metadata'sı grafikle uyuşmuyor"));
        }
        Ok(())
    }

    fn tensor_bul(&self, id: u64, islem: &str) -> Result<TensorData, String> {
        self.nodes
            .get(&id)
            .cloned()
            .ok_or_else(|| format!("{islem}: tensor grafikte bulunamadı: {id}"))
    }

    fn veri_kopyala(
        tensor: &TensorData,
        expected_size: usize,
        islem: &str,
    ) -> Result<Vec<f64>, String> {
        let values = tensor
            .veri
            .lock()
            .map_err(|_| format!("{islem}: tensor veri kilidi bozuldu"))?;
        if values.len() != expected_size {
            return Err(format!(
                "{islem}: tensor veri uzunluğu bozuk; {expected_size} bekleniyordu, {} bulundu",
                values.len()
            ));
        }
        Self::sonlu_dogrula(&values, islem)?;
        Ok(values.clone())
    }
}

pub static AUTOGRAD_GRAF: once_cell::sync::Lazy<Arc<Mutex<AutogradGraph>>> =
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(AutogradGraph::new())));

#[cfg(test)]
mod tests {
    use super::AutogradGraph;

    #[test]
    fn geri_yayilim_tekrarlaninca_onceki_turden_artik_biriktirmez() {
        let mut graph = AutogradGraph::new();
        let left = graph
            .tensor_olustur(1, 1, vec![2.0], true)
            .expect("Sol tensor oluşturulmalı");
        let right = graph
            .tensor_olustur(1, 1, vec![3.0], true)
            .expect("Sağ tensor oluşturulmalı");
        let output = graph.topla(&left, &right).expect("Toplama çalışmalı");

        graph
            .backward(output.id)
            .expect("İlk geri yayılım çalışmalı");
        graph
            .backward(output.id)
            .expect("İkinci geri yayılım çalışmalı");
        assert_eq!(
            *left.gradyan.lock().expect("Gradyan kilidi alınmalı"),
            vec![1.0]
        );
        assert_eq!(
            *right.gradyan.lock().expect("Gradyan kilidi alınmalı"),
            vec![1.0]
        );
    }

    #[test]
    fn geri_yayilim_commit_oncesi_hata_onceki_gradyanlari_degistirmez() {
        let mut graph = AutogradGraph::new();
        let left = graph
            .tensor_olustur(1, 1, vec![2.0], true)
            .expect("Sol tensor oluşturulmalı");
        let right = graph
            .tensor_olustur(1, 1, vec![3.0], true)
            .expect("Sağ tensor oluşturulmalı");
        let output = graph.topla(&left, &right).expect("Toplama çalışmalı");
        *left.gradyan.lock().expect("Sol gradyan kilidi alınmalı") = vec![9.0];
        right
            .gradyan
            .lock()
            .expect("Sağ gradyan kilidi alınmalı")
            .clear();

        assert!(graph.backward(output.id).is_err());
        assert_eq!(
            *left.gradyan.lock().expect("Gradyan kilidi alınmalı"),
            vec![9.0]
        );
        assert_eq!(
            *output.gradyan.lock().expect("Çıktı gradyanı alınmalı"),
            vec![0.0]
        );
    }

    #[test]
    fn gradyan_gerektirmeyen_hedef_geri_yayilimi_reddeder() {
        let mut graph = AutogradGraph::new();
        let tensor = graph
            .tensor_olustur(1, 1, vec![2.0], false)
            .expect("Tensor oluşturulmalı");
        assert!(graph.backward(tensor.id).is_err());
    }
}
