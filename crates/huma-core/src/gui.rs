use crate::interpreter::Yorumlayici;
use crate::value::{BuiltinRuntime, Deger};
use eframe::egui;
use std::cell::RefCell;
use std::collections::HashMap;

thread_local! {
    static CURRENT_UI: RefCell<Option<*mut egui::Ui>> = const { RefCell::new(None) };
    static GUI_REQUEST: RefCell<Option<GuiRequest>> = const { RefCell::new(None) };
}

const MAX_WINDOW_SIZE: f64 = 16_384.0;
const MAX_TEXT_BYTES: usize = 1024 * 1024;

struct GuiRequest {
    baslik: String,
    genislik: f32,
    yukseklik: f32,
    cizim_fks: Deger,
}

struct UiContextGuard {
    previous: Option<*mut egui::Ui>,
}

impl UiContextGuard {
    fn enter(current: *mut egui::Ui) -> Result<Self, String> {
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

fn with_ui<T>(operation: &str, action: impl FnOnce(&mut egui::Ui) -> T) -> Result<T, String> {
    let pointer = CURRENT_UI.with(|cell| {
        cell.try_borrow()
            .map_err(|_| format!("{operation}: GUI bağlamı kullanımda"))
            .and_then(|slot| {
                (*slot).ok_or_else(|| {
                    format!("{operation}: yalnızca pencere çizim fonksiyonu içinde kullanılabilir")
                })
            })
    })?;
    // İşaretçi yalnızca `UiContextGuard` ömrü boyunca, aynı iş parçacığında
    // kurulur. Geri çağrıdan önce iç UI için yeni bir guard oluşturulur.
    Ok(unsafe { action(&mut *pointer) })
}

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

fn checked_text(text: &str, operation: &str) -> Result<(), String> {
    if text.len() > MAX_TEXT_BYTES {
        Err(format!(
            "{operation}: metin {MAX_TEXT_BYTES} bayt sınırını aşıyor"
        ))
    } else {
        Ok(())
    }
}

fn nested_call(
    runtime: &mut dyn BuiltinRuntime,
    function: &Deger,
    ui: &mut egui::Ui,
    operation: &str,
) -> Deger {
    let _guard = match UiContextGuard::enter(ui as *mut egui::Ui) {
        Ok(guard) => guard,
        Err(error) => return Deger::Hata(format!("{operation}: {error}")),
    };
    runtime.call_value(function.clone(), Vec::new())
}

pub struct HumaGuiApp {
    cizim_fks: Deger,
    interp: Yorumlayici,
}

impl HumaGuiApp {
    pub fn new(cizim_fks: Deger, interp: Yorumlayici) -> Self {
        Self { cizim_fks, interp }
    }
}

impl eframe::App for HumaGuiApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            let _guard = match UiContextGuard::enter(ui as *mut egui::Ui) {
                Ok(guard) => guard,
                Err(error) => {
                    ui.colored_label(egui::Color32::RED, error);
                    return;
                }
            };
            if let Deger::Hata(error) = self
                .interp
                .fonksiyon_cagrisi(self.cizim_fks.clone(), Vec::new())
            {
                ui.colored_label(egui::Color32::RED, error);
            }
        });
    }
}

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
                    (text, Some(egui::Color32::from_rgb(red, green, blue)), None)
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
                    (
                        text,
                        Some(egui::Color32::from_rgb(red, green, blue)),
                        Some([width, height]),
                    )
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
            match with_ui("buton", |ui| {
                let mut rich = egui::RichText::new(text);
                if let Some(color) = color {
                    rich = rich.color(color);
                }
                let button = egui::Button::new(rich);
                let response = match size {
                    Some(size) => ui.add_sized(size, button),
                    None => ui.add(button),
                };
                response.clicked()
            }) {
                Ok(clicked) => Deger::Sayi(if clicked { 1.0 } else { 0.0 }),
                Err(error) => Deger::Hata(error),
            }
        }),
    );

    globals.insert(
        "tema_ayarla".to_string(),
        Deger::DahiliFonksiyon(|args| match args.as_slice() {
            [Deger::Metin(theme)] if theme == "koyu" || theme == "açık" => {
                match with_ui("tema_ayarla", |ui| {
                    if theme == "koyu" {
                        ui.ctx().set_visuals(egui::Visuals::dark());
                    } else {
                        ui.ctx().set_visuals(egui::Visuals::light());
                    }
                }) {
                    Ok(()) => Deger::Sayi(1.0),
                    Err(error) => Deger::Hata(error),
                }
            }
            [Deger::Metin(_)] => {
                Deger::Hata("tema_ayarla: tema 'koyu' veya 'açık' olmalıdır".to_string())
            }
            [_] => Deger::Hata("tema_ayarla: tema metin olmalıdır".to_string()),
            _ => Deger::Hata(format!(
                "tema_ayarla: tam olarak 1 argüman bekleniyordu; {} geldi",
                args.len()
            )),
        }),
    );

    globals.insert(
        "girdi_alanı".to_string(),
        Deger::DahiliFonksiyon(|args| {
            let (mut text, width) = match args.as_slice() {
                [Deger::Metin(text)] => (text.clone(), None),
                [Deger::Metin(text), Deger::Sayi(width)] => {
                    let width = match finite_f32(*width, "girdi_alanı genişlik", true) {
                        Ok(value) => value,
                        Err(error) => return Deger::Hata(error),
                    };
                    (text.clone(), Some(width))
                }
                _ => {
                    return Deger::Hata(
                        "girdi_alanı: metin ve isteğe bağlı pozitif genişlik gerekir".to_string(),
                    )
                }
            };
            if let Err(error) = checked_text(&text, "girdi_alanı") {
                return Deger::Hata(error);
            }
            match with_ui("girdi_alanı", |ui| {
                let edit = egui::TextEdit::singleline(&mut text);
                match width {
                    Some(width) => {
                        ui.add_sized([width, 20.0], edit);
                    }
                    None => {
                        ui.add(edit);
                    }
                }
            }) {
                Ok(()) if text.len() <= MAX_TEXT_BYTES => Deger::Metin(text),
                Ok(()) => Deger::Hata("girdi_alanı: metin sınırı aşıldı".to_string()),
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
            let mut text = text.clone();
            match with_ui("büyük_girdi_alanı", |ui| {
                ui.add(egui::TextEdit::multiline(&mut text));
            }) {
                Ok(()) if text.len() <= MAX_TEXT_BYTES => Deger::Metin(text),
                Ok(()) => Deger::Hata("büyük_girdi_alanı: metin sınırı aşıldı".to_string()),
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
            let mut value = *value;
            match with_ui("kaydırıcı", |ui| {
                ui.add(egui::Slider::new(&mut value, *minimum..=*maximum));
            }) {
                Ok(()) => Deger::Sayi(value),
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
                ui.checkbox(&mut checked, text);
            }) {
                Ok(()) => Deger::Sayi(if checked { 1.0 } else { 0.0 }),
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
            match with_ui("ayraç", |ui| ui.separator()) {
                Ok(_) => Deger::Bos,
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
            match with_ui("boşluk", |ui| ui.add_space(amount)) {
                Ok(()) => Deger::Bos,
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
            match with_ui("sekme", |ui| ui.selectable_label(selected, text).clicked()) {
                Ok(clicked) => Deger::Sayi(if clicked { 1.0 } else { 0.0 }),
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
                let context = ui.ctx().clone();
                egui::Window::new(title)
                    .open(&mut open)
                    .show(&context, |inner| {
                        result = nested_call(runtime, draw, inner, "yüzen_pencere");
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

    globals.insert(
        "satır_bitir".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if !args.is_empty() {
                return Deger::Hata(format!(
                    "satır_bitir: argüman beklenmiyordu; {} geldi",
                    args.len()
                ));
            }
            match with_ui("satır_bitir", |ui| ui.end_row()) {
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
            match with_ui("alan_ayır", |ui| {
                ui.allocate_ui(egui::Vec2::new(width, height), |inner| {
                    result = nested_call(runtime, draw, inner, "alan_ayır");
                });
            }) {
                Err(error) => Deger::Hata(error),
                Ok(()) => result,
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
                Some(egui::Color32::from_rgb(red, green, blue)),
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
                    (text, None, Some(egui::Color32::from_rgb(red, green, blue)))
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
                let mut rich = egui::RichText::new(text);
                if let Some(color) = color {
                    rich = rich.color(color);
                }
                match style {
                    Some("kalın") => {
                        ui.label(rich.strong());
                    }
                    Some("eğik") => {
                        ui.label(rich.italics());
                    }
                    Some("başlık") => {
                        ui.heading(rich);
                    }
                    _ => {
                        ui.label(rich);
                    }
                }
            }) {
                Ok(()) => Deger::Bos,
                Err(error) => Deger::Hata(error),
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

fn execute_callback_layout(
    runtime: &mut dyn BuiltinRuntime,
    args: Vec<Deger>,
    name: &str,
    layout: impl FnOnce(&mut egui::Ui, &mut dyn FnMut(&mut egui::Ui)),
) -> Deger {
    let [draw] = args.as_slice() else {
        return Deger::Hata(format!("{name}: tam olarak 1 geri çağrı gerekir"));
    };
    if let Err(error) = callback(draw, name) {
        return Deger::Hata(error);
    }
    let mut result = Deger::Bos;
    match with_ui(name, |ui| {
        let mut invoke = |inner: &mut egui::Ui| {
            result = nested_call(runtime, draw, inner, name);
        };
        layout(ui, &mut invoke);
    }) {
        Err(error) => Deger::Hata(error),
        Ok(()) => result,
    }
}

fn callback_layout_horizontal(runtime: &mut dyn BuiltinRuntime, args: Vec<Deger>) -> Deger {
    execute_callback_layout(runtime, args, "yan_yana", |ui, draw| {
        ui.horizontal(draw);
    })
}

fn callback_layout_vertical(runtime: &mut dyn BuiltinRuntime, args: Vec<Deger>) -> Deger {
    execute_callback_layout(runtime, args, "alt_alta", |ui, draw| {
        ui.vertical(draw);
    })
}

fn callback_layout_menu_bar(runtime: &mut dyn BuiltinRuntime, args: Vec<Deger>) -> Deger {
    execute_callback_layout(runtime, args, "menü_çubuğu", |ui, draw| {
        egui::MenuBar::new().ui(ui, draw);
    })
}

fn register_named_callback_layout(
    globals: &mut HashMap<String, Deger>,
    name: &'static str,
    function: fn(&mut dyn BuiltinRuntime, Vec<Deger>) -> Deger,
) {
    globals.insert(name.to_string(), Deger::BaglamliDahiliFonksiyon(function));
}

fn execute_named_layout(
    runtime: &mut dyn BuiltinRuntime,
    args: Vec<Deger>,
    name: &str,
    layout: impl FnOnce(&mut egui::Ui, &str, &mut dyn FnMut(&mut egui::Ui)),
) -> Deger {
    let [Deger::Metin(identifier), draw] = args.as_slice() else {
        return Deger::Hata(format!("{name}: metin adı ve geri çağrı gerekir"));
    };
    if let Err(error) = checked_text(identifier, name) {
        return Deger::Hata(error);
    }
    if let Err(error) = callback(draw, name) {
        return Deger::Hata(error);
    }
    let mut result = Deger::Bos;
    match with_ui(name, |ui| {
        let mut invoke = |inner: &mut egui::Ui| {
            result = nested_call(runtime, draw, inner, name);
        };
        layout(ui, identifier, &mut invoke);
    }) {
        Err(error) => Deger::Hata(error),
        Ok(()) => result,
    }
}

fn named_layout_menu(runtime: &mut dyn BuiltinRuntime, args: Vec<Deger>) -> Deger {
    execute_named_layout(runtime, args, "açılır_menü", |ui, name, draw| {
        ui.menu_button(name, draw);
    })
}

fn named_layout_group(runtime: &mut dyn BuiltinRuntime, args: Vec<Deger>) -> Deger {
    execute_named_layout(runtime, args, "grup_kutusu", |ui, name, draw| {
        ui.group(|inner| {
            inner.label(name);
            inner.separator();
            draw(inner);
        });
    })
}

fn named_layout_grid(runtime: &mut dyn BuiltinRuntime, args: Vec<Deger>) -> Deger {
    execute_named_layout(runtime, args, "grid_oluştur", |ui, name, draw| {
        egui::Grid::new(name).striped(true).show(ui, draw);
    })
}

fn named_layout_scroll(runtime: &mut dyn BuiltinRuntime, args: Vec<Deger>) -> Deger {
    execute_named_layout(runtime, args, "kaydırılabilir_alan", |ui, name, draw| {
        egui::ScrollArea::vertical()
            .id_salt(name)
            .auto_shrink([false, false])
            .show(ui, draw);
    })
}

fn sized_button(
    text: &str,
    width: f64,
    height: f64,
    color: Option<egui::Color32>,
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
        let mut rich = egui::RichText::new(text);
        if let Some(color) = color {
            rich = rich.color(color);
        }
        ui.add_sized([width, height], egui::Button::new(rich))
            .clicked()
    }) {
        Ok(clicked) => Deger::Sayi(if clicked { 1.0 } else { 0.0 }),
        Err(error) => Deger::Hata(error),
    }
}

pub fn gui_istegi_var_mi() -> Result<bool, String> {
    GUI_REQUEST.with(|request| {
        request
            .try_borrow()
            .map(|request| request.is_some())
            .map_err(|_| "GUI isteği durumu kullanımda".to_string())
    })
}

pub fn gui_calistir(interp: Yorumlayici) -> Result<(), String> {
    let request = GUI_REQUEST.with(|request| {
        request
            .try_borrow_mut()
            .map_err(|_| "GUI isteği kullanımda".to_string())
            .map(|mut request| request.take())
    })?;
    let request = request.ok_or_else(|| "Başlatılacak GUI isteği yok".to_string())?;
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([request.genislik, request.yukseklik]),
        ..Default::default()
    };
    let title = request.baslik;
    let draw = request.cizim_fks;
    eframe::run_native(
        &title.clone(),
        options,
        Box::new(|_creation_context| Ok(Box::new(HumaGuiApp::new(draw, interp)))),
    )
    .map_err(|error| format!("GUI çalıştırılamadı: {error}"))
}
