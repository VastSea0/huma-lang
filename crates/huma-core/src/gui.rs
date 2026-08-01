//! Hüma Native GUI — Dear ImGui (dear-imgui-rs + dear-app) tabanlı yerel arayüz katmanı.
//!
//! Bu modül, egui yerine Dear ImGui (immediate-mode, endüstri standardı, hafif) kullanır.
//! Hüma tarafındaki `çizim_fks` her karede yeniden çağrılır; widget'lar güncel değerlerini
//! okuyup döndürür (immediate-mode). Bu mimari, önceki egui tabanlı sürümle bire bir
//! aynı çağrı sözleşmesini (her karede çağrılan tek bir çizim fonksiyonu) korur.
//!
//! Kişiselleştirme: `tema_ayarla` / `tema_olustur` / `tema_listele` ile hazır tema
//! paleti ve tamamen özelleştirilebilir bir tema sistemi sunulur (bkz. [`HumaTheme`]).

use crate::interpreter::Yorumlayici;
use crate::value::{BuiltinRuntime, Deger};
use dear_app::{AppBuilder, RunnerConfig};
use dear_imgui_rs::{
    Condition, FontId, FontSource, StyleColor, StyleTweaks, TabBar, TabItem, Theme as ImTheme,
    ThemePreset, TreeNodeFlags, Ui, WindowFlags,
};
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

// ---------------------------------------------------------------------------
// Gömülü yazı tipleri (Türkçe karakter desteği için Noto Sans; SIL OFL 1.1)
// ---------------------------------------------------------------------------

const FONT_REGULAR: &[u8] = include_bytes!("../../../assets/fonts/NotoSans-Regular.ttf");
const FONT_BOLD: &[u8] = include_bytes!("../../../assets/fonts/NotoSans-Bold.ttf");
const FONT_ITALIC: &[u8] = include_bytes!("../../../assets/fonts/NotoSans-Italic.ttf");

#[derive(Clone, Copy)]
struct FontHandles {
    body: FontId,
    bold: FontId,
    italic: FontId,
    heading: FontId,
}

thread_local! {
    static CURRENT_UI: RefCell<Option<*const Ui>> = const { RefCell::new(None) };
    static GUI_REQUEST: RefCell<Option<GuiRequest>> = const { RefCell::new(None) };
    static AUTO_ID: Cell<i32> = const { Cell::new(0) };
    static LAYOUT_STACK: RefCell<Vec<LayoutKind>> = const { RefCell::new(Vec::new()) };
    static FONT_HANDLES: RefCell<Option<FontHandles>> = const { RefCell::new(None) };
}

const MAX_WINDOW_SIZE: f64 = 16_384.0;
const MAX_TEXT_BYTES: usize = 1024 * 1024;

struct GuiRequest {
    baslik: String,
    genislik: f32,
    yukseklik: f32,
    cizim_fks: Deger,
}

enum LayoutKind {
    Horizontal { first: bool },
    Vertical,
}

// ---------------------------------------------------------------------------
// UI bağlamı yönetimi
// ---------------------------------------------------------------------------

struct UiContextGuard {
    previous: Option<*const Ui>,
}

impl UiContextGuard {
    fn enter(current: *const Ui) -> Result<Self, String> {
        CURRENT_UI.with(|cell| {
            let mut slot = cell
                .try_borrow_mut()
                .map_err(|_| "GUI bağlamı kullanımda".to_string())?;
            let previous = slot.replace(current);
            Ok(Self { previous })
        })
    }
}

impl Drop for UiContextGuard {
    fn drop(&mut self) {
        CURRENT_UI.with(|cell| {
            if let Ok(mut slot) = cell.try_borrow_mut() {
                *slot = self.previous;
            }
        });
    }
}

fn with_ui<T>(operation: &str, action: impl FnOnce(&Ui) -> T) -> Result<T, String> {
    let pointer = CURRENT_UI.with(|cell| {
        cell.try_borrow()
            .map_err(|_| format!("{operation}: GUI bağlamı kullanımda"))
            .and_then(|slot| {
                (*slot).ok_or_else(|| {
                    format!("{operation}: yalnızca pencere çizim fonksiyonu içinde kullanılabilir")
                })
            })
    })?;
    // İşaretçi yalnızca `UiContextGuard` ömrü boyunca, aynı iş parçacığında kurulur ve
    // kare süresince (iç içe konteynerler dahil) geçerliliğini korur.
    Ok(unsafe { action(&*pointer) })
}

fn next_auto_id() -> i32 {
    AUTO_ID.with(|c| {
        let id = c.get();
        c.set(id + 1);
        id
    })
}

fn push_layout(kind: LayoutKind) {
    LAYOUT_STACK.with(|s| s.borrow_mut().push(kind));
}

fn pop_layout() {
    LAYOUT_STACK.with(|s| {
        s.borrow_mut().pop();
    });
}

/// Yatay (`yan_yana_diz`) ya da ızgara (`grid_oluştur`) bağlamındaysa, bir sonraki
/// widget'ı otomatik olarak aynı satıra yerleştirir. Script'in elle `same_line`
/// çağırmasına gerek kalmaz.
fn before_widget(ui: &Ui) {
    LAYOUT_STACK.with(|stack| {
        if let Some(LayoutKind::Horizontal { first }) = stack.borrow_mut().last_mut() {
            if *first {
                *first = false;
            } else {
                ui.same_line();
            }
        }
    });
}

// ---------------------------------------------------------------------------
// Doğrulama yardımcıları
// ---------------------------------------------------------------------------

fn is_callable(value: &Deger) -> bool {
    matches!(
        value,
        Deger::Fonksiyon { .. }
            | Deger::BytecodeFonksiyon { .. }
            | Deger::DahiliFonksiyon(_)
            | Deger::BaglamliDahiliFonksiyon(_)
    )
}

fn callback<'a>(value: &'a Deger, operation: &str) -> Result<&'a Deger, String> {
    if is_callable(value) {
        Ok(value)
    } else {
        Err(format!("{operation}: geri çağrı fonksiyon olmalıdır"))
    }
}

fn finite_f32(value: f64, operation: &str, positive: bool) -> Result<f32, String> {
    let valid_sign = if positive { value > 0.0 } else { value >= 0.0 };
    if !value.is_finite() || !valid_sign || value > MAX_WINDOW_SIZE {
        let qualifier = if positive {
            "pozitif"
        } else {
            "negatif olmayan"
        };
        return Err(format!(
            "{operation}: değer {qualifier}, sonlu ve en fazla {MAX_WINDOW_SIZE} olmalıdır"
        ));
    }
    Ok(value as f32)
}

fn color_component(value: f64, operation: &str) -> Result<u8, String> {
    if !value.is_finite() || value.fract() != 0.0 || !(0.0..=255.0).contains(&value) {
        return Err(format!(
            "{operation}: renk bileşenleri 0..255 aralığında tamsayı olmalıdır"
        ));
    }
    Ok(value as u8)
}

fn binary_flag(value: f64, operation: &str) -> Result<bool, String> {
    match value {
        0.0 => Ok(false),
        1.0 => Ok(true),
        _ => Err(format!("{operation}: durum bayrağı 0 veya 1 olmalıdır")),
    }
}

fn unit_interval(value: f64, operation: &str) -> Result<f32, String> {
    if !value.is_finite() || !(0.0..=1.0).contains(&value) {
        return Err(format!("{operation}: değer 0.0 ile 1.0 arasında olmalıdır"));
    }
    Ok(value as f32)
}

fn checked_text(text: &str, operation: &str) -> Result<(), String> {
    if text.len() > MAX_TEXT_BYTES {
        Err(format!(
            "{operation}: metin {MAX_TEXT_BYTES} bayt sınırını aşıyor"
        ))
    } else {
        Ok(())
    }
}

fn rgba_of(color: (u8, u8, u8), alpha: f32) -> [f32; 4] {
    [
        color.0 as f32 / 255.0,
        color.1 as f32 / 255.0,
        color.2 as f32 / 255.0,
        alpha,
    ]
}

fn lighten(color: (u8, u8, u8), amount: f32) -> (u8, u8, u8) {
    let f = |v: u8| -> u8 {
        (v as f32 + (255.0 - v as f32) * amount)
            .round()
            .clamp(0.0, 255.0) as u8
    };
    (f(color.0), f(color.1), f(color.2))
}

fn darken(color: (u8, u8, u8), amount: f32) -> (u8, u8, u8) {
    let f = |v: u8| -> u8 { (v as f32 * (1.0 - amount)).round().clamp(0.0, 255.0) as u8 };
    (f(color.0), f(color.1), f(color.2))
}

// ---------------------------------------------------------------------------
// Tema sistemi (kişiselleştirme)
// ---------------------------------------------------------------------------

/// Kullanıcının seçtiği/oluşturduğu, uygulanabilir yüksek seviyeli tema tanımı.
#[derive(Clone, Copy)]
struct HumaTheme {
    karanlik: bool,
    aksan: (u8, u8, u8),
    kose: f32,
    aralik: f32,
}

const TEMA_ADLARI: &[&str] = &[
    "koyu",
    "açık",
    "gece_mavisi",
    "mor_alacakaranlık",
    "orman",
    "gün_batımı",
    "okyanus",
    "kiraz",
    "mono",
];

fn preset_theme(name: &str) -> Option<HumaTheme> {
    Some(match name {
        "koyu" => HumaTheme {
            karanlik: true,
            aksan: (66, 133, 244),
            kose: 6.0,
            aralik: 8.0,
        },
        "açık" => HumaTheme {
            karanlik: false,
            aksan: (25, 103, 210),
            kose: 6.0,
            aralik: 8.0,
        },
        "gece_mavisi" => HumaTheme {
            karanlik: true,
            aksan: (88, 101, 242),
            kose: 8.0,
            aralik: 8.0,
        },
        "mor_alacakaranlık" => HumaTheme {
            karanlik: true,
            aksan: (168, 85, 247),
            kose: 10.0,
            aralik: 8.0,
        },
        "orman" => HumaTheme {
            karanlik: true,
            aksan: (52, 168, 83),
            kose: 6.0,
            aralik: 8.0,
        },
        "gün_batımı" => HumaTheme {
            karanlik: true,
            aksan: (255, 138, 61),
            kose: 10.0,
            aralik: 9.0,
        },
        "okyanus" => HumaTheme {
            karanlik: true,
            aksan: (20, 184, 196),
            kose: 8.0,
            aralik: 8.0,
        },
        "kiraz" => HumaTheme {
            karanlik: true,
            aksan: (236, 72, 110),
            kose: 10.0,
            aralik: 8.0,
        },
        "mono" => HumaTheme {
            karanlik: true,
            aksan: (160, 160, 160),
            kose: 4.0,
            aralik: 6.0,
        },
        _ => return None,
    })
}

fn accent_overrides(accent: (u8, u8, u8)) -> Vec<dear_imgui_rs::ColorOverride> {
    use dear_imgui_rs::ColorOverride;
    let base = rgba_of(accent, 1.0);
    let hover = rgba_of(lighten(accent, 0.12), 1.0);
    let active = rgba_of(darken(accent, 0.12), 1.0);
    let soft = rgba_of(accent, 0.32);
    let strong_dark = rgba_of(darken(accent, 0.35), 1.0);
    vec![
        ColorOverride {
            id: StyleColor::Button,
            rgba: soft,
        },
        ColorOverride {
            id: StyleColor::ButtonHovered,
            rgba: base,
        },
        ColorOverride {
            id: StyleColor::ButtonActive,
            rgba: active,
        },
        ColorOverride {
            id: StyleColor::CheckMark,
            rgba: base,
        },
        ColorOverride {
            id: StyleColor::CheckboxSelectedBg,
            rgba: soft,
        },
        ColorOverride {
            id: StyleColor::SliderGrab,
            rgba: base,
        },
        ColorOverride {
            id: StyleColor::SliderGrabActive,
            rgba: active,
        },
        ColorOverride {
            id: StyleColor::Header,
            rgba: soft,
        },
        ColorOverride {
            id: StyleColor::HeaderHovered,
            rgba: base,
        },
        ColorOverride {
            id: StyleColor::HeaderActive,
            rgba: active,
        },
        ColorOverride {
            id: StyleColor::TabSelected,
            rgba: base,
        },
        ColorOverride {
            id: StyleColor::TitleBgActive,
            rgba: strong_dark,
        },
        ColorOverride {
            id: StyleColor::FrameBgActive,
            rgba: soft,
        },
        ColorOverride {
            id: StyleColor::FrameBgHovered,
            rgba: [hover[0], hover[1], hover[2], 0.35],
        },
        ColorOverride {
            id: StyleColor::SeparatorActive,
            rgba: base,
        },
        ColorOverride {
            id: StyleColor::ResizeGripActive,
            rgba: base,
        },
        ColorOverride {
            id: StyleColor::TextLink,
            rgba: base,
        },
        ColorOverride {
            id: StyleColor::DockingPreview,
            rgba: soft,
        },
    ]
}

fn build_imgui_theme(theme: HumaTheme) -> ImTheme {
    ImTheme {
        preset: if theme.karanlik {
            ThemePreset::Dark
        } else {
            ThemePreset::Light
        },
        colors: accent_overrides(theme.aksan),
        style: StyleTweaks {
            window_rounding: Some(theme.kose),
            frame_rounding: Some((theme.kose * 0.7).max(2.0)),
            popup_rounding: Some((theme.kose * 0.6).max(2.0)),
            child_rounding: Some((theme.kose * 0.6).max(2.0)),
            tab_rounding: Some((theme.kose * 0.7).max(2.0)),
            scrollbar_rounding: Some(theme.kose),
            grab_rounding: Some(theme.kose),
            item_spacing: Some([theme.aralik, (theme.aralik * 0.7).max(2.0)]),
            window_padding: Some([theme.aralik, theme.aralik]),
            frame_padding: Some([(theme.aralik * 0.7).max(2.0), (theme.aralik * 0.5).max(2.0)]),
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Şu anki karede (frame) aktif olan Dear ImGui stiline temayı doğrudan uygular.
fn apply_theme_now(theme: HumaTheme) -> Result<(), String> {
    with_ui("tema_ayarla", |_ui| {
        // Style, aktif bağlamın ImGuiStyle yapısına dönüştürülebilir bir görünümdür;
        // `dear_imgui_rs::Style` bu yapıyla bellek düzeni uyumlu (transparent) bir sarmalayıcıdır.
        let style: &mut dear_imgui_rs::Style =
            unsafe { &mut *(dear_imgui_rs::sys::igGetStyle() as *mut dear_imgui_rs::Style) };
        build_imgui_theme(theme).apply_to_style(style);
    })
}

fn dict_number(map: &HashMap<String, Deger>, key: &str, operation: &str) -> Result<f64, String> {
    match map.get(key) {
        Some(Deger::Sayi(n)) => Ok(*n),
        _ => Err(format!(
            "{operation}: tema tanımında sayısal '{key}' alanı gerekir"
        )),
    }
}

fn theme_from_dict(map: &HashMap<String, Deger>, operation: &str) -> Result<HumaTheme, String> {
    let karanlik = binary_flag(dict_number(map, "karanlık", operation)?, operation)?;
    let r = color_component(dict_number(map, "r", operation)?, operation)?;
    let g = color_component(dict_number(map, "g", operation)?, operation)?;
    let b = color_component(dict_number(map, "b", operation)?, operation)?;
    let kose = finite_f32(dict_number(map, "köşe", operation)?, operation, false)?;
    let aralik = finite_f32(dict_number(map, "aralık", operation)?, operation, false)?;
    Ok(HumaTheme {
        karanlik,
        aksan: (r, g, b),
        kose,
        aralik,
    })
}

fn theme_to_dict(theme: HumaTheme) -> Deger {
    let mut map = HashMap::new();
    map.insert(
        "karanlık".to_string(),
        Deger::Sayi(if theme.karanlik { 1.0 } else { 0.0 }),
    );
    map.insert("r".to_string(), Deger::Sayi(theme.aksan.0 as f64));
    map.insert("g".to_string(), Deger::Sayi(theme.aksan.1 as f64));
    map.insert("b".to_string(), Deger::Sayi(theme.aksan.2 as f64));
    map.insert("köşe".to_string(), Deger::Sayi(theme.kose as f64));
    map.insert("aralık".to_string(), Deger::Sayi(theme.aralik as f64));
    Deger::Sozluk(Rc::new(RefCell::new(map)))
}

// ---------------------------------------------------------------------------
// Font yardımcıları
// ---------------------------------------------------------------------------

fn push_style_font<'ui>(
    ui: &'ui Ui,
    style: Option<&str>,
) -> Option<dear_imgui_rs::FontStackToken<'ui>> {
    let style = style?;
    FONT_HANDLES.with(|cell| {
        let handles = cell.borrow();
        let handles = handles.as_ref()?;
        let id = match style {
            "kalın" => handles.bold,
            "eğik" => handles.italic,
            "başlık" => handles.heading,
            _ => handles.body,
        };
        Some(ui.push_font(id))
    })
}

// ---------------------------------------------------------------------------
// Buton çizim yardımcıları
// ---------------------------------------------------------------------------

fn button_color_tokens<'ui>(
    ui: &'ui Ui,
    color: Option<(u8, u8, u8)>,
) -> Vec<dear_imgui_rs::ColorStackToken<'ui>> {
    let Some(color) = color else {
        return Vec::new();
    };
    let base = rgba_of(color, 0.85);
    let hover = rgba_of(lighten(color, 0.1), 1.0);
    let active = rgba_of(darken(color, 0.1), 1.0);
    vec![
        ui.push_style_color(StyleColor::Button, base),
        ui.push_style_color(StyleColor::ButtonHovered, hover),
        ui.push_style_color(StyleColor::ButtonActive, active),
        ui.push_style_color(StyleColor::Text, [1.0, 1.0, 1.0, 1.0]),
    ]
}

fn draw_button(ui: &Ui, text: &str, size: Option<[f32; 2]>, color: Option<(u8, u8, u8)>) -> bool {
    before_widget(ui);
    let _id = ui.push_id(next_auto_id());
    let _colors = button_color_tokens(ui, color);
    match size {
        Some(size) => ui.button_config(text).size(size).build(),
        None => ui.button(text),
    }
}

fn sized_button(
    text: &str,
    width: f64,
    height: f64,
    color: Option<(u8, u8, u8)>,
    operation: &str,
) -> Deger {
    if let Err(error) = checked_text(text, operation) {
        return Deger::Hata(error);
    }
    let width = match finite_f32(width, &format!("{operation} genişlik"), true) {
        Ok(value) => value,
        Err(error) => return Deger::Hata(error),
    };
    let height = match finite_f32(height, &format!("{operation} yükseklik"), true) {
        Ok(value) => value,
        Err(error) => return Deger::Hata(error),
    };
    match with_ui(operation, |ui| {
        draw_button(ui, text, Some([width, height]), color)
    }) {
        Ok(clicked) => Deger::Sayi(if clicked { 1.0 } else { 0.0 }),
        Err(error) => Deger::Hata(error),
    }
}

// ---------------------------------------------------------------------------
// Kayıt
// ---------------------------------------------------------------------------

pub fn kayit_et(globals: &mut HashMap<String, Deger>) {
    if crate::capability::require(crate::capability::Capability::Gui, "GUI yerleşikleri").is_err()
    {
        return;
    }

    globals.insert(
        "pencere_başlat".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let [Deger::Metin(title), Deger::Sayi(width), Deger::Sayi(height), draw] =
                args.as_slice()
            else {
                return if args.len() == 4 {
                    Deger::Hata(
                        "pencere_başlat: başlık, genişlik, yükseklik ve çizim fonksiyonu gerekir"
                            .to_string(),
                    )
                } else {
                    Deger::Hata(format!(
                        "pencere_başlat: tam olarak 4 argüman bekleniyordu; {} geldi",
                        args.len()
                    ))
                };
            };
            if let Err(error) = checked_text(title, "pencere_başlat") {
                return Deger::Hata(error);
            }
            let width = match finite_f32(*width, "pencere_başlat genişlik", true) {
                Ok(value) => value,
                Err(error) => return Deger::Hata(error),
            };
            let height = match finite_f32(*height, "pencere_başlat yükseklik", true) {
                Ok(value) => value,
                Err(error) => return Deger::Hata(error),
            };
            if let Err(error) = callback(draw, "pencere_başlat") {
                return Deger::Hata(error);
            }
            GUI_REQUEST.with(|request| {
                let mut request = match request.try_borrow_mut() {
                    Ok(request) => request,
                    Err(_) => {
                        return Deger::Hata("pencere_başlat: GUI isteği kullanımda".to_string())
                    }
                };
                if request.is_some() {
                    return Deger::Hata(
                        "pencere_başlat: aynı çalıştırmada yalnızca bir pencere başlatılabilir"
                            .to_string(),
                    );
                }
                *request = Some(GuiRequest {
                    baslik: title.clone(),
                    genislik: width,
                    yukseklik: height,
                    cizim_fks: draw.clone(),
                });
                Deger::Sayi(1.0)
            })
        }),
    );

    globals.insert(
        "buton".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let (text, color, size) = match args.as_slice() {
                [Deger::Metin(text)] => (text, None, None),
                [Deger::Metin(text), Deger::Sayi(width), Deger::Sayi(height)] => {
                    let width = match finite_f32(*width, "buton genişlik", true) {
                        Ok(value) => value,
                        Err(error) => return Deger::Hata(error),
                    };
                    let height = match finite_f32(*height, "buton yükseklik", true) {
                        Ok(value) => value,
                        Err(error) => return Deger::Hata(error),
                    };
                    (text, None, Some([width, height]))
                }
                [Deger::Metin(text), Deger::Sayi(red), Deger::Sayi(green), Deger::Sayi(blue)] => {
                    let red = match color_component(*red, "buton") {
                        Ok(value) => value,
                        Err(error) => return Deger::Hata(error),
                    };
                    let green = match color_component(*green, "buton") {
                        Ok(value) => value,
                        Err(error) => return Deger::Hata(error),
                    };
                    let blue = match color_component(*blue, "buton") {
                        Ok(value) => value,
                        Err(error) => return Deger::Hata(error),
                    };
                    (text, Some((red, green, blue)), None)
                }
                [Deger::Metin(text), Deger::Sayi(red), Deger::Sayi(green), Deger::Sayi(blue), Deger::Sayi(width), Deger::Sayi(height)] => {
                    let red = match color_component(*red, "buton") {
                        Ok(value) => value,
                        Err(error) => return Deger::Hata(error),
                    };
                    let green = match color_component(*green, "buton") {
                        Ok(value) => value,
                        Err(error) => return Deger::Hata(error),
                    };
                    let blue = match color_component(*blue, "buton") {
                        Ok(value) => value,
                        Err(error) => return Deger::Hata(error),
                    };
                    let width = match finite_f32(*width, "buton genişlik", true) {
                        Ok(value) => value,
                        Err(error) => return Deger::Hata(error),
                    };
                    let height = match finite_f32(*height, "buton yükseklik", true) {
                        Ok(value) => value,
                        Err(error) => return Deger::Hata(error),
                    };
                    (text, Some((red, green, blue)), Some([width, height]))
                }
                _ => {
                    return Deger::Hata(
                        "buton: desteklenen imzalar (metin), (metin,w,h), (metin,r,g,b) ve (metin,r,g,b,w,h)"
                            .to_string(),
                    )
                }
            };
            if let Err(error) = checked_text(text, "buton") {
                return Deger::Hata(error);
            }
            match with_ui("buton", |ui| draw_button(ui, text, size, color)) {
                Ok(clicked) => Deger::Sayi(if clicked { 1.0 } else { 0.0 }),
                Err(error) => Deger::Hata(error),
            }
        }),
    );

    globals.insert(
        "boyutlu_buton".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let [Deger::Metin(text), Deger::Sayi(width), Deger::Sayi(height)] = args.as_slice()
            else {
                return Deger::Hata(
                    "boyutlu_buton: metin, genişlik ve yükseklik gerekir".to_string(),
                );
            };
            sized_button(text, *width, *height, None, "boyutlu_buton")
        }),
    );

    globals.insert(
        "boyutlu_renkli_buton".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let [Deger::Metin(text), Deger::Sayi(width), Deger::Sayi(height), Deger::Sayi(red), Deger::Sayi(green), Deger::Sayi(blue)] =
                args.as_slice()
            else {
                return Deger::Hata(
                    "boyutlu_renkli_buton: metin, genişlik, yükseklik ve RGB gerekir".to_string(),
                );
            };
            let red = match color_component(*red, "boyutlu_renkli_buton") {
                Ok(value) => value,
                Err(error) => return Deger::Hata(error),
            };
            let green = match color_component(*green, "boyutlu_renkli_buton") {
                Ok(value) => value,
                Err(error) => return Deger::Hata(error),
            };
            let blue = match color_component(*blue, "boyutlu_renkli_buton") {
                Ok(value) => value,
                Err(error) => return Deger::Hata(error),
            };
            sized_button(
                text,
                *width,
                *height,
                Some((red, green, blue)),
                "boyutlu_renkli_buton",
            )
        }),
    );

    globals.insert(
        "etiket".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let (text, style, color) = match args.as_slice() {
                [Deger::Metin(text)] => (text, None, None),
                [Deger::Metin(text), Deger::Metin(style)]
                    if matches!(style.as_str(), "kalın" | "eğik" | "başlık") =>
                {
                    (text, Some(style.as_str()), None)
                }
                [Deger::Metin(_), Deger::Metin(_)] => {
                    return Deger::Hata(
                        "etiket: stil 'kalın', 'eğik' veya 'başlık' olmalıdır".to_string(),
                    )
                }
                [Deger::Metin(text), Deger::Sayi(red), Deger::Sayi(green), Deger::Sayi(blue)] => {
                    let red = match color_component(*red, "etiket") {
                        Ok(value) => value,
                        Err(error) => return Deger::Hata(error),
                    };
                    let green = match color_component(*green, "etiket") {
                        Ok(value) => value,
                        Err(error) => return Deger::Hata(error),
                    };
                    let blue = match color_component(*blue, "etiket") {
                        Ok(value) => value,
                        Err(error) => return Deger::Hata(error),
                    };
                    (text, None, Some((red, green, blue)))
                }
                _ => {
                    return Deger::Hata(
                        "etiket: desteklenen imzalar (metin), (metin,stil) ve (metin,r,g,b)"
                            .to_string(),
                    )
                }
            };
            if let Err(error) = checked_text(text, "etiket") {
                return Deger::Hata(error);
            }
            match with_ui("etiket", |ui| {
                before_widget(ui);
                let _font = push_style_font(ui, style);
                match color {
                    Some(color) => ui.text_colored(rgba_of(color, 1.0), text),
                    None => ui.text(text),
                }
            }) {
                Ok(()) => Deger::Bos,
                Err(error) => Deger::Hata(error),
            }
        }),
    );

    globals.insert(
        "tema_ayarla".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let theme = match args.as_slice() {
                [Deger::Metin(name)] => match preset_theme(name) {
                    Some(theme) => theme,
                    None => {
                        return Deger::Hata(format!(
                            "tema_ayarla: bilinmeyen tema '{name}'; mevcut temalar için tema_listele() kullanın"
                        ))
                    }
                },
                [Deger::Sozluk(map)] => match theme_from_dict(&map.borrow(), "tema_ayarla") {
                    Ok(theme) => theme,
                    Err(error) => return Deger::Hata(error),
                },
                _ => {
                    return Deger::Hata(
                        "tema_ayarla: tema adı (metin) ya da tema_olustur() ile üretilmiş bir tema bekleniyor"
                            .to_string(),
                    )
                }
            };
            match apply_theme_now(theme) {
                Ok(()) => Deger::Sayi(1.0),
                Err(error) => Deger::Hata(error),
            }
        }),
    );

    globals.insert(
        "tema_olustur".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let [Deger::Sayi(karanlik), Deger::Sayi(red), Deger::Sayi(green), Deger::Sayi(blue), Deger::Sayi(kose), Deger::Sayi(aralik)] =
                args.as_slice()
            else {
                return Deger::Hata(
                    "tema_olustur: karanlık_mı(0/1), r, g, b, köşe_yuvarlama, aralık gerekir"
                        .to_string(),
                );
            };
            let karanlik = match binary_flag(*karanlik, "tema_olustur") {
                Ok(value) => value,
                Err(error) => return Deger::Hata(error),
            };
            let red = match color_component(*red, "tema_olustur") {
                Ok(value) => value,
                Err(error) => return Deger::Hata(error),
            };
            let green = match color_component(*green, "tema_olustur") {
                Ok(value) => value,
                Err(error) => return Deger::Hata(error),
            };
            let blue = match color_component(*blue, "tema_olustur") {
                Ok(value) => value,
                Err(error) => return Deger::Hata(error),
            };
            let kose = match finite_f32(*kose, "tema_olustur köşe yuvarlama", false) {
                Ok(value) => value,
                Err(error) => return Deger::Hata(error),
            };
            let aralik = match finite_f32(*aralik, "tema_olustur aralık", false) {
                Ok(value) => value,
                Err(error) => return Deger::Hata(error),
            };
            theme_to_dict(HumaTheme {
                karanlik,
                aksan: (red, green, blue),
                kose,
                aralik,
            })
        }),
    );

    globals.insert(
        "tema_listele".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if !args.is_empty() {
                return Deger::Hata(format!(
                    "tema_listele: argüman beklenmiyordu; {} geldi",
                    args.len()
                ));
            }
            let list = TEMA_ADLARI
                .iter()
                .map(|name| Deger::Metin(name.to_string()))
                .collect();
            Deger::Liste(Rc::new(RefCell::new(list)))
        }),
    );

    globals.insert(
        "girdi_alanı".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let (text, width) = match args.as_slice() {
                [Deger::Metin(text)] => (text, None),
                [Deger::Metin(text), Deger::Sayi(width)] => {
                    let width = match finite_f32(*width, "girdi_alanı genişlik", true) {
                        Ok(value) => value,
                        Err(error) => return Deger::Hata(error),
                    };
                    (text, Some(width))
                }
                _ => {
                    return Deger::Hata(
                        "girdi_alanı: metin ve isteğe bağlı pozitif genişlik gerekir".to_string(),
                    )
                }
            };
            if let Err(error) = checked_text(text, "girdi_alanı") {
                return Deger::Hata(error);
            }
            match with_ui("girdi_alanı", |ui| {
                before_widget(ui);
                let _id = ui.push_id(next_auto_id());
                if let Some(width) = width {
                    ui.set_next_item_width(width);
                }
                let mut buf = text.clone();
                ui.input_text("##huma_girdi", &mut buf).build();
                buf
            }) {
                Ok(buf) if buf.len() <= MAX_TEXT_BYTES => Deger::Metin(buf),
                Ok(_) => Deger::Hata("girdi_alanı: metin sınırı aşıldı".to_string()),
                Err(error) => Deger::Hata(error),
            }
        }),
    );

    globals.insert(
        "büyük_girdi_alanı".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let [Deger::Metin(text)] = args.as_slice() else {
                return Deger::Hata(
                    "büyük_girdi_alanı: tam olarak 1 metin argümanı gerekir".to_string(),
                );
            };
            if let Err(error) = checked_text(text, "büyük_girdi_alanı") {
                return Deger::Hata(error);
            }
            match with_ui("büyük_girdi_alanı", |ui| {
                before_widget(ui);
                let _id = ui.push_id(next_auto_id());
                let mut buf = text.clone();
                ui.input_text_multiline("##huma_coklu_girdi", &mut buf, [0.0, 150.0])
                    .build();
                buf
            }) {
                Ok(buf) if buf.len() <= MAX_TEXT_BYTES => Deger::Metin(buf),
                Ok(_) => Deger::Hata("büyük_girdi_alanı: metin sınırı aşıldı".to_string()),
                Err(error) => Deger::Hata(error),
            }
        }),
    );

    globals.insert(
        "kaydırıcı".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let [Deger::Sayi(value), Deger::Sayi(minimum), Deger::Sayi(maximum)] = args.as_slice()
            else {
                return Deger::Hata("kaydırıcı: değer, alt sınır ve üst sınır gerekir".to_string());
            };
            if !value.is_finite()
                || !minimum.is_finite()
                || !maximum.is_finite()
                || minimum > maximum
                || value < minimum
                || value > maximum
            {
                return Deger::Hata(
                    "kaydırıcı: sonlu değer alt ve üst sınır arasında olmalıdır".to_string(),
                );
            }
            let mut fvalue = *value as f32;
            let (fmin, fmax) = (*minimum as f32, *maximum as f32);
            match with_ui("kaydırıcı", |ui| {
                before_widget(ui);
                let _id = ui.push_id(next_auto_id());
                ui.slider_f32("##huma_kaydirici", &mut fvalue, fmin, fmax);
            }) {
                Ok(()) => Deger::Sayi(fvalue as f64),
                Err(error) => Deger::Hata(error),
            }
        }),
    );

    globals.insert(
        "onay_kutusu".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let [Deger::Sayi(state), Deger::Metin(text)] = args.as_slice() else {
                return Deger::Hata("onay_kutusu: 0/1 durum ve metin gerekir".to_string());
            };
            let mut checked = match binary_flag(*state, "onay_kutusu") {
                Ok(value) => value,
                Err(error) => return Deger::Hata(error),
            };
            if let Err(error) = checked_text(text, "onay_kutusu") {
                return Deger::Hata(error);
            }
            match with_ui("onay_kutusu", |ui| {
                before_widget(ui);
                let _id = ui.push_id(next_auto_id());
                ui.checkbox(text, &mut checked);
            }) {
                Ok(()) => Deger::Sayi(if checked { 1.0 } else { 0.0 }),
                Err(error) => Deger::Hata(error),
            }
        }),
    );

    globals.insert(
        "radyo_düğmesi".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let [Deger::Sayi(active), Deger::Metin(text)] = args.as_slice() else {
                return Deger::Hata("radyo_düğmesi: 0/1 seçili durumu ve metin gerekir".to_string());
            };
            let active = match binary_flag(*active, "radyo_düğmesi") {
                Ok(value) => value,
                Err(error) => return Deger::Hata(error),
            };
            if let Err(error) = checked_text(text, "radyo_düğmesi") {
                return Deger::Hata(error);
            }
            match with_ui("radyo_düğmesi", |ui| {
                before_widget(ui);
                let _id = ui.push_id(next_auto_id());
                ui.radio_button_bool(text, active)
            }) {
                Ok(clicked) => Deger::Sayi(if clicked { 1.0 } else { 0.0 }),
                Err(error) => Deger::Hata(error),
            }
        }),
    );

    globals.insert(
        "sekme".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let [Deger::Sayi(selected), Deger::Metin(text)] = args.as_slice() else {
                return Deger::Hata("sekme: 0/1 seçim durumu ve metin gerekir".to_string());
            };
            let selected = match binary_flag(*selected, "sekme") {
                Ok(value) => value,
                Err(error) => return Deger::Hata(error),
            };
            match with_ui("sekme", |ui| {
                before_widget(ui);
                let _id = ui.push_id(next_auto_id());
                ui.selectable_config(text).selected(selected).build()
            }) {
                Ok(clicked) => Deger::Sayi(if clicked { 1.0 } else { 0.0 }),
                Err(error) => Deger::Hata(error),
            }
        }),
    );

    globals.insert(
        "ilerleme_çubuğu".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let (fraction, label) = match args.as_slice() {
                [Deger::Sayi(fraction)] => (fraction, None),
                [Deger::Sayi(fraction), Deger::Metin(label)] => (fraction, Some(label)),
                _ => {
                    return Deger::Hata(
                        "ilerleme_çubuğu: 0.0-1.0 arası değer ve isteğe bağlı etiket gerekir"
                            .to_string(),
                    )
                }
            };
            let fraction = match unit_interval(*fraction, "ilerleme_çubuğu") {
                Ok(value) => value,
                Err(error) => return Deger::Hata(error),
            };
            match with_ui("ilerleme_çubuğu", |ui| {
                before_widget(ui);
                let _id = ui.push_id(next_auto_id());
                let bar = ui.progress_bar(fraction);
                match label {
                    Some(label) => bar.overlay_text(label.as_str()).build(),
                    None => bar.build(),
                }
            }) {
                Ok(()) => Deger::Bos,
                Err(error) => Deger::Hata(error),
            }
        }),
    );

    globals.insert(
        "açılır_liste".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let [Deger::Sayi(selected), Deger::Liste(options)] = args.as_slice() else {
                return Deger::Hata(
                    "açılır_liste: seçili indeks ve metin listesi gerekir".to_string(),
                );
            };
            let options_ref = options.borrow();
            let mut string_options = Vec::with_capacity(options_ref.len());
            for item in options_ref.iter() {
                match item {
                    Deger::Metin(text) => string_options.push(text.clone()),
                    _ => return Deger::Hata("açılır_liste: liste yalnızca metin içermelidir".to_string()),
                }
            }
            drop(options_ref);
            if string_options.is_empty() {
                return Deger::Hata("açılır_liste: seçenek listesi boş olamaz".to_string());
            }
            if !selected.is_finite() || *selected < 0.0 || selected.fract() != 0.0 {
                return Deger::Hata("açılır_liste: seçili indeks negatif olmayan bir tamsayı olmalıdır".to_string());
            }
            let mut index = (*selected as usize).min(string_options.len() - 1);
            match with_ui("açılır_liste", |ui| {
                before_widget(ui);
                let _id = ui.push_id(next_auto_id());
                ui.combo_simple_string("##huma_acilir_liste", &mut index, &string_options);
            }) {
                Ok(()) => Deger::Sayi(index as f64),
                Err(error) => Deger::Hata(error),
            }
        }),
    );

    globals.insert(
        "renk_seçici".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let [Deger::Sayi(red), Deger::Sayi(green), Deger::Sayi(blue)] = args.as_slice() else {
                return Deger::Hata("renk_seçici: r, g, b (0..255) gerekir".to_string());
            };
            let red = match color_component(*red, "renk_seçici") {
                Ok(value) => value,
                Err(error) => return Deger::Hata(error),
            };
            let green = match color_component(*green, "renk_seçici") {
                Ok(value) => value,
                Err(error) => return Deger::Hata(error),
            };
            let blue = match color_component(*blue, "renk_seçici") {
                Ok(value) => value,
                Err(error) => return Deger::Hata(error),
            };
            let mut rgb = [
                red as f32 / 255.0,
                green as f32 / 255.0,
                blue as f32 / 255.0,
            ];
            match with_ui("renk_seçici", |ui| {
                before_widget(ui);
                let _id = ui.push_id(next_auto_id());
                ui.color_edit3("##huma_renk_secici", &mut rgb);
            }) {
                Ok(()) => {
                    let values = vec![
                        Deger::Sayi((rgb[0] * 255.0).round() as f64),
                        Deger::Sayi((rgb[1] * 255.0).round() as f64),
                        Deger::Sayi((rgb[2] * 255.0).round() as f64),
                    ];
                    Deger::Liste(Rc::new(RefCell::new(values)))
                }
                Err(error) => Deger::Hata(error),
            }
        }),
    );

    globals.insert(
        "bağlantı".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let [Deger::Metin(text), Deger::Metin(url)] = args.as_slice() else {
                return Deger::Hata("bağlantı: görünen metin ve URL gerekir".to_string());
            };
            if let Err(error) = checked_text(text, "bağlantı") {
                return Deger::Hata(error);
            }
            match with_ui("bağlantı", |ui| {
                before_widget(ui);
                let _id = ui.push_id(next_auto_id());
                ui.text_link_open_url(text, url)
            }) {
                Ok(clicked) => Deger::Sayi(if clicked { 1.0 } else { 0.0 }),
                Err(error) => Deger::Hata(error),
            }
        }),
    );

    globals.insert(
        "menü_ögesi".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let [Deger::Metin(text)] = args.as_slice() else {
                return Deger::Hata("menü_ögesi: tam olarak 1 metin argümanı gerekir".to_string());
            };
            match with_ui("menü_ögesi", |ui| ui.menu_item(text)) {
                Ok(clicked) => Deger::Sayi(if clicked { 1.0 } else { 0.0 }),
                Err(error) => Deger::Hata(error),
            }
        }),
    );

    register_callback_layout(globals, "yan_yana", callback_layout_horizontal);
    register_callback_layout(globals, "alt_alta", callback_layout_vertical);
    register_callback_layout(globals, "menü_çubuğu", callback_layout_menu_bar);

    globals.insert(
        "ayraç".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if !args.is_empty() {
                return Deger::Hata(format!(
                    "ayraç: argüman beklenmiyordu; {} geldi",
                    args.len()
                ));
            }
            match with_ui("ayraç", |ui| {
                before_widget(ui);
                ui.separator();
            }) {
                Ok(()) => Deger::Bos,
                Err(error) => Deger::Hata(error),
            }
        }),
    );

    globals.insert(
        "boşluk".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let [Deger::Sayi(amount)] = args.as_slice() else {
                return Deger::Hata("boşluk: tam olarak 1 sayı argümanı gerekir".to_string());
            };
            let amount = match finite_f32(*amount, "boşluk", false) {
                Ok(value) => value,
                Err(error) => return Deger::Hata(error),
            };
            match with_ui("boşluk", |ui| {
                before_widget(ui);
                ui.dummy([amount, amount]);
            }) {
                Ok(()) => Deger::Bos,
                Err(error) => Deger::Hata(error),
            }
        }),
    );

    globals.insert(
        "yüzen_pencere".to_string(),
        Deger::BaglamliDahiliFonksiyon(|runtime, args| {
            let [Deger::Metin(title), Deger::Sayi(open), draw] = args.as_slice() else {
                return Deger::Hata(
                    "yüzen_pencere: başlık, 0/1 açık durumu ve geri çağrı gerekir".to_string(),
                );
            };
            let mut open = match binary_flag(*open, "yüzen_pencere") {
                Ok(value) => value,
                Err(error) => return Deger::Hata(error),
            };
            if let Err(error) = callback(draw, "yüzen_pencere") {
                return Deger::Hata(error);
            }
            let mut result = Deger::Bos;
            let shown = with_ui("yüzen_pencere", |ui| {
                ui.window(title.as_str())
                    .opened(&mut open)
                    .size([340.0, 220.0], Condition::FirstUseEver)
                    .build(|| {
                        result = runtime.call_value(draw.clone(), Vec::new());
                    });
            });
            match shown {
                Err(error) => Deger::Hata(error),
                Ok(()) if matches!(result, Deger::Hata(_)) => result,
                Ok(()) => Deger::Sayi(if open { 1.0 } else { 0.0 }),
            }
        }),
    );

    register_named_callback_layout(globals, "açılır_menü", named_layout_menu);
    register_named_callback_layout(globals, "grup_kutusu", named_layout_group);
    register_named_callback_layout(globals, "grid_oluştur", named_layout_grid);
    register_named_callback_layout(globals, "kaydırılabilir_alan", named_layout_scroll);
    register_named_callback_layout(globals, "sekme_grubu", named_layout_tab_bar);
    register_named_callback_layout(globals, "sekme_sayfası", named_layout_tab_item);
    register_named_callback_layout(globals, "ağaç_düğümü", named_layout_tree_node);

    globals.insert(
        "kart".to_string(),
        Deger::BaglamliDahiliFonksiyon(|runtime, args| {
            let [draw] = args.as_slice() else {
                return Deger::Hata("kart: tam olarak 1 geri çağrı gerekir".to_string());
            };
            if let Err(error) = callback(draw, "kart") {
                return Deger::Hata(error);
            }
            let mut result = Deger::Bos;
            let outcome = with_ui("kart", |ui| {
                before_widget(ui);
                let id = format!("huma_kart##{}", next_auto_id());
                ui.child_window(id).border(true).size([0.0, 0.0]).build(ui, || {
                    result = runtime.call_value(draw.clone(), Vec::new());
                });
            });
            match outcome {
                Err(error) => Deger::Hata(error),
                Ok(()) => result,
            }
        }),
    );

    globals.insert(
        "satır_bitir".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if !args.is_empty() {
                return Deger::Hata(format!(
                    "satır_bitir: argüman beklenmiyordu; {} geldi",
                    args.len()
                ));
            }
            match with_ui("satır_bitir", |ui| {
                ui.new_line();
                LAYOUT_STACK.with(|stack| {
                    if let Some(LayoutKind::Horizontal { first }) = stack.borrow_mut().last_mut() {
                        *first = true;
                    }
                });
            }) {
                Ok(()) => Deger::Bos,
                Err(error) => Deger::Hata(error),
            }
        }),
    );

    globals.insert(
        "alan_ayır".to_string(),
        Deger::BaglamliDahiliFonksiyon(|runtime, args| {
            let [Deger::Sayi(width), Deger::Sayi(height), draw] = args.as_slice() else {
                return Deger::Hata(
                    "alan_ayır: genişlik, yükseklik ve geri çağrı gerekir".to_string(),
                );
            };
            let width = match finite_f32(*width, "alan_ayır genişlik", true) {
                Ok(value) => value,
                Err(error) => return Deger::Hata(error),
            };
            let height = match finite_f32(*height, "alan_ayır yükseklik", true) {
                Ok(value) => value,
                Err(error) => return Deger::Hata(error),
            };
            if let Err(error) = callback(draw, "alan_ayır") {
                return Deger::Hata(error);
            }
            let mut result = Deger::Bos;
            let outcome = with_ui("alan_ayır", |ui| {
                before_widget(ui);
                let id = format!("huma_alan##{}", next_auto_id());
                ui.child_window(id)
                    .border(false)
                    .size([width, height])
                    .build(ui, || {
                        result = runtime.call_value(draw.clone(), Vec::new());
                    });
            });
            match outcome {
                Err(error) => Deger::Hata(error),
                Ok(()) => result,
            }
        }),
    );
}

fn register_callback_layout(
    globals: &mut HashMap<String, Deger>,
    name: &'static str,
    function: fn(&mut dyn BuiltinRuntime, Vec<Deger>) -> Deger,
) {
    globals.insert(name.to_string(), Deger::BaglamliDahiliFonksiyon(function));
}

fn callback_layout_horizontal(runtime: &mut dyn BuiltinRuntime, args: Vec<Deger>) -> Deger {
    let [draw] = args.as_slice() else {
        return Deger::Hata("yan_yana: tam olarak 1 geri çağrı gerekir".to_string());
    };
    if let Err(error) = callback(draw, "yan_yana") {
        return Deger::Hata(error);
    }
    if let Err(error) = with_ui("yan_yana", before_widget) {
        return Deger::Hata(error);
    }
    push_layout(LayoutKind::Horizontal { first: true });
    let result = runtime.call_value(draw.clone(), Vec::new());
    pop_layout();
    result
}

fn callback_layout_vertical(runtime: &mut dyn BuiltinRuntime, args: Vec<Deger>) -> Deger {
    let [draw] = args.as_slice() else {
        return Deger::Hata("alt_alta: tam olarak 1 geri çağrı gerekir".to_string());
    };
    if let Err(error) = callback(draw, "alt_alta") {
        return Deger::Hata(error);
    }
    if let Err(error) = with_ui("alt_alta", before_widget) {
        return Deger::Hata(error);
    }
    push_layout(LayoutKind::Vertical);
    let result = runtime.call_value(draw.clone(), Vec::new());
    pop_layout();
    result
}

fn callback_layout_menu_bar(runtime: &mut dyn BuiltinRuntime, args: Vec<Deger>) -> Deger {
    let [draw] = args.as_slice() else {
        return Deger::Hata("menü_çubuğu: tam olarak 1 geri çağrı gerekir".to_string());
    };
    if let Err(error) = callback(draw, "menü_çubuğu") {
        return Deger::Hata(error);
    }
    let mut result = Deger::Bos;
    let outcome = with_ui("menü_çubuğu", |ui| {
        if let Some(_token) = ui.begin_menu_bar() {
            result = runtime.call_value(draw.clone(), Vec::new());
        }
    });
    match outcome {
        Err(error) => Deger::Hata(error),
        Ok(()) => result,
    }
}

fn register_named_callback_layout(
    globals: &mut HashMap<String, Deger>,
    name: &'static str,
    function: fn(&mut dyn BuiltinRuntime, Vec<Deger>) -> Deger,
) {
    globals.insert(name.to_string(), Deger::BaglamliDahiliFonksiyon(function));
}

fn named_layout_menu(runtime: &mut dyn BuiltinRuntime, args: Vec<Deger>) -> Deger {
    let [Deger::Metin(title), draw] = args.as_slice() else {
        return Deger::Hata("açılır_menü: metin adı ve geri çağrı gerekir".to_string());
    };
    if let Err(error) = callback(draw, "açılır_menü") {
        return Deger::Hata(error);
    }
    let mut result = Deger::Bos;
    let outcome = with_ui("açılır_menü", |ui| {
        ui.menu(title, || {
            result = runtime.call_value(draw.clone(), Vec::new());
        });
    });
    match outcome {
        Err(error) => Deger::Hata(error),
        Ok(()) => result,
    }
}

fn named_layout_group(runtime: &mut dyn BuiltinRuntime, args: Vec<Deger>) -> Deger {
    let [Deger::Metin(title), draw] = args.as_slice() else {
        return Deger::Hata("grup_kutusu: metin adı ve geri çağrı gerekir".to_string());
    };
    if let Err(error) = callback(draw, "grup_kutusu") {
        return Deger::Hata(error);
    }
    let mut result = Deger::Bos;
    let outcome = with_ui("grup_kutusu", |ui| {
        before_widget(ui);
        let id = format!("{title}##huma_grup{}", next_auto_id());
        ui.child_window(id).border(true).size([0.0, 0.0]).build(ui, || {
            ui.text(title);
            ui.separator();
            result = runtime.call_value(draw.clone(), Vec::new());
        });
    });
    match outcome {
        Err(error) => Deger::Hata(error),
        Ok(()) => result,
    }
}

fn named_layout_grid(runtime: &mut dyn BuiltinRuntime, args: Vec<Deger>) -> Deger {
    let [Deger::Metin(_id), draw] = args.as_slice() else {
        return Deger::Hata("grid_oluştur: metin adı ve geri çağrı gerekir".to_string());
    };
    if let Err(error) = callback(draw, "grid_oluştur") {
        return Deger::Hata(error);
    }
    if let Err(error) = with_ui("grid_oluştur", before_widget) {
        return Deger::Hata(error);
    }
    push_layout(LayoutKind::Horizontal { first: true });
    let result = runtime.call_value(draw.clone(), Vec::new());
    pop_layout();
    result
}

fn named_layout_scroll(runtime: &mut dyn BuiltinRuntime, args: Vec<Deger>) -> Deger {
    let [Deger::Metin(id), draw] = args.as_slice() else {
        return Deger::Hata("kaydırılabilir_alan: metin adı ve geri çağrı gerekir".to_string());
    };
    if let Err(error) = callback(draw, "kaydırılabilir_alan") {
        return Deger::Hata(error);
    }
    let mut result = Deger::Bos;
    let outcome = with_ui("kaydırılabilir_alan", |ui| {
        before_widget(ui);
        let unique = format!("{id}##huma_kaydir{}", next_auto_id());
        ui.child_window(unique)
            .border(true)
            .size([0.0, 0.0])
            .build(ui, || {
                result = runtime.call_value(draw.clone(), Vec::new());
            });
    });
    match outcome {
        Err(error) => Deger::Hata(error),
        Ok(()) => result,
    }
}

fn named_layout_tab_bar(runtime: &mut dyn BuiltinRuntime, args: Vec<Deger>) -> Deger {
    let [Deger::Metin(id), draw] = args.as_slice() else {
        return Deger::Hata("sekme_grubu: metin adı ve geri çağrı gerekir".to_string());
    };
    if let Err(error) = callback(draw, "sekme_grubu") {
        return Deger::Hata(error);
    }
    let mut result = Deger::Bos;
    let outcome = with_ui("sekme_grubu", |ui| {
        before_widget(ui);
        TabBar::new(id.as_str()).build(ui, || {
            result = runtime.call_value(draw.clone(), Vec::new());
        });
    });
    match outcome {
        Err(error) => Deger::Hata(error),
        Ok(()) => result,
    }
}

fn named_layout_tab_item(runtime: &mut dyn BuiltinRuntime, args: Vec<Deger>) -> Deger {
    let [Deger::Metin(title), draw] = args.as_slice() else {
        return Deger::Hata("sekme_sayfası: başlık ve geri çağrı gerekir; yalnızca sekme_grubu içinde kullanılabilir".to_string());
    };
    if let Err(error) = callback(draw, "sekme_sayfası") {
        return Deger::Hata(error);
    }
    let mut result = Deger::Bos;
    let outcome = with_ui("sekme_sayfası", |ui| {
        TabItem::new(title.as_str()).build(ui, || {
            result = runtime.call_value(draw.clone(), Vec::new());
        });
    });
    match outcome {
        Err(error) => Deger::Hata(error),
        Ok(()) => result,
    }
}

fn named_layout_tree_node(runtime: &mut dyn BuiltinRuntime, args: Vec<Deger>) -> Deger {
    let [Deger::Metin(title), draw] = args.as_slice() else {
        return Deger::Hata("ağaç_düğümü: başlık ve geri çağrı gerekir".to_string());
    };
    if let Err(error) = callback(draw, "ağaç_düğümü") {
        return Deger::Hata(error);
    }
    let opened = with_ui("ağaç_düğümü", |ui| {
        before_widget(ui);
        let _id = ui.push_id(next_auto_id());
        ui.collapsing_header(title, TreeNodeFlags::empty())
    });
    match opened {
        Err(error) => Deger::Hata(error),
        Ok(true) => runtime.call_value(draw.clone(), Vec::new()),
        Ok(false) => Deger::Bos,
    }
}

// ---------------------------------------------------------------------------
// Çalıştırma
// ---------------------------------------------------------------------------

pub fn gui_istegi_var_mi() -> Result<bool, String> {
    GUI_REQUEST.with(|request| {
        request
            .try_borrow()
            .map(|request| request.is_some())
            .map_err(|_| "GUI isteği durumu kullanımda".to_string())
    })
}

fn load_fonts(ctx: &mut dear_imgui_rs::Context) {
    let mut atlas = ctx.fonts();
    let body = atlas.add_font(&[FontSource::ttf_data_with_size(FONT_REGULAR, 18.0)]);
    let bold = atlas.add_font(&[FontSource::ttf_data_with_size(FONT_BOLD, 18.0)]);
    let italic = atlas.add_font(&[FontSource::ttf_data_with_size(FONT_ITALIC, 18.0)]);
    let heading = atlas.add_font(&[FontSource::ttf_data_with_size(FONT_BOLD, 26.0)]);
    FONT_HANDLES.with(|cell| {
        *cell.borrow_mut() = Some(FontHandles {
            body,
            bold,
            italic,
            heading,
        });
    });
}

pub fn gui_calistir(interp: Yorumlayici) -> Result<(), String> {
    let request = GUI_REQUEST.with(|request| {
        request
            .try_borrow_mut()
            .map_err(|_| "GUI isteği kullanımda".to_string())
            .map(|mut request| request.take())
    })?;
    let request = request.ok_or_else(|| "Başlatılacak GUI isteği yok".to_string())?;

    let runner = RunnerConfig {
        window_title: request.baslik,
        window_size: (request.genislik as f64, request.yukseklik as f64),
        ..Default::default()
    };

    let draw_fn = request.cizim_fks;
    let mut interp = interp;

    AppBuilder::new()
        .with_config(runner)
        .on_fonts(load_fonts)
        .on_style(|ctx| {
            let style = ctx.style_mut();
            let default_theme = preset_theme("gece_mavisi").expect("varsayılan tema mevcut olmalı");
            build_imgui_theme(default_theme).apply_to_style(style);
        })
        .on_frame(move |ui, _addons| {
            AUTO_ID.with(|c| c.set(0));
            let _guard = match UiContextGuard::enter(ui as *const Ui) {
                Ok(guard) => guard,
                Err(_) => return,
            };
            let display_size = ui.io().display_size();
            let flags = WindowFlags::NO_TITLE_BAR
                | WindowFlags::NO_RESIZE
                | WindowFlags::NO_MOVE
                | WindowFlags::NO_COLLAPSE
                | WindowFlags::NO_SAVED_SETTINGS
                | WindowFlags::NO_BRING_TO_FRONT_ON_FOCUS
                | WindowFlags::NO_NAV_FOCUS
                | WindowFlags::MENU_BAR;
            ui.window("##huma_kok_penceresi")
                .position([0.0, 0.0], Condition::Always)
                .size(display_size, Condition::Always)
                .flags(flags)
                .build(|| {
                    if let Deger::Hata(error) =
                        interp.fonksiyon_cagrisi(draw_fn.clone(), Vec::new())
                    {
                        ui.text_colored([1.0, 0.35, 0.35, 1.0], &error);
                    }
                });
        })
        .run()
        .map_err(|error| format!("GUI çalıştırılamadı: {error}"))
}
