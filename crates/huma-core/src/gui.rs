use crate::interpreter::Yorumlayici;
use crate::value::Deger;
use eframe::egui;
use std::cell::RefCell;

thread_local! {
    static CURRENT_UI: RefCell<Option<*mut egui::Ui>> = const { RefCell::new(None) };
    static GUI_REQUEST: RefCell<Option<GuiRequest>> = const { RefCell::new(None) };
    static CURRENT_INTERP: RefCell<Option<*mut Yorumlayici>> = const { RefCell::new(None) };
}

struct GuiRequest {
    baslik: String,
    genislik: f32,
    yukseklik: f32,
    cizim_fks: Deger,
}

pub struct HumaGuiApp {
    cizim_fks: Deger,
    interp: Yorumlayici,
}

impl HumaGuiApp {
    pub fn new(_baslik: String, cizim_fks: Deger, interp: Yorumlayici) -> Self {
        // GUI fonksiyonlarını tekrar eklememize gerek yok, zaten varlar.
        Self { cizim_fks, interp }
    }
}

impl eframe::App for HumaGuiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let ui_ptr = ui as *mut egui::Ui;
            CURRENT_UI.with(|c| *c.borrow_mut() = Some(ui_ptr));

            let interp_ptr = &mut self.interp as *mut Yorumlayici;
            CURRENT_INTERP.with(|i| *i.borrow_mut() = Some(interp_ptr));

            // Hüma çizim fonksiyonunu çağır
            self.interp
                .fonksiyon_cagrisi(self.cizim_fks.clone(), vec![]);

            CURRENT_INTERP.with(|i| *i.borrow_mut() = None);
            CURRENT_UI.with(|c| *c.borrow_mut() = None);
        });
    }
}

pub fn kayit_et(globals: &mut std::collections::HashMap<String, Deger>) {
    globals.insert(
        "pencere_başlat".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if args.len() < 4 {
                return Deger::Bos;
            }
            let baslik = args[0].to_string();
            let genislik = if let Deger::Sayi(n) = args[1] {
                n as f32
            } else {
                800.0
            };
            let yukseklik = if let Deger::Sayi(n) = args[2] {
                n as f32
            } else {
                600.0
            };
            let cizim_fks = args[3].clone();

            GUI_REQUEST.with(|r| {
                *r.borrow_mut() = Some(GuiRequest {
                    baslik,
                    genislik,
                    yukseklik,
                    cizim_fks,
                });
            });

            Deger::Bos
        }),
    );

    globals.insert(
        "buton".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if let Some(metin) = args.first() {
                let res = CURRENT_UI.with(|c| {
                    if let Some(ui_ptr) = *c.borrow() {
                        let ui = unsafe { &mut *ui_ptr };

                        let mut rich = egui::RichText::new(metin.to_string());
                        let mut w_h: Option<(f32, f32)> = None;

                        // Argüman analiz sistemi:
                        // buton(metin) -> [0]
                        // buton(metin, r, g, b) -> [0..4]
                        // buton(metin, w, h) -> [0..3]
                        // buton(metin, r, g, b, w, h) -> [0..6]

                        if args.len() == 3 {
                            // Sadece boyut
                            if let (Deger::Sayi(w), Deger::Sayi(h)) = (&args[1], &args[2]) {
                                w_h = Some((*w as f32, *h as f32));
                            }
                        } else if args.len() >= 4 {
                            // Renk var
                            if let (Deger::Sayi(r), Deger::Sayi(g), Deger::Sayi(b)) =
                                (&args[1], &args[2], &args[3])
                            {
                                rich = rich
                                    .color(egui::Color32::from_rgb(*r as u8, *g as u8, *b as u8));
                            }
                            if args.len() >= 6 {
                                // Renk + Boyut
                                if let (Deger::Sayi(w), Deger::Sayi(h)) = (&args[4], &args[5]) {
                                    w_h = Some((*w as f32, *h as f32));
                                }
                            }
                        }

                        let btn = egui::Button::new(rich);
                        let response = if let Some((w, h)) = w_h {
                            ui.add_sized([w, h], btn)
                        } else {
                            ui.add(btn)
                        };

                        if response.clicked() {
                            return 1.0;
                        }
                    }
                    0.0
                });
                return Deger::Sayi(res);
            }
            Deger::Sayi(0.0)
        }),
    );

    globals.insert(
        "tema_ayarla".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if let Some(Deger::Metin(tema)) = args.first() {
                CURRENT_UI.with(|c| {
                    if let Some(ui_ptr) = *c.borrow() {
                        let ui = unsafe { &mut *ui_ptr };
                        if tema == "koyu" {
                            ui.ctx().set_visuals(egui::Visuals::dark());
                        } else if tema == "açık" {
                            ui.ctx().set_visuals(egui::Visuals::light());
                        }
                    }
                });
            }
            Deger::Bos
        }),
    );

    globals.insert(
        "girdi_alanı".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if let Some(Deger::Metin(mut text)) = args.first().cloned() {
                CURRENT_UI.with(|c| {
                    if let Some(ui_ptr) = *c.borrow() {
                        let ui = unsafe { &mut *ui_ptr };
                        let edit = egui::TextEdit::singleline(&mut text);

                        if args.len() >= 2 {
                            if let Deger::Sayi(w) = args[1] {
                                ui.add_sized([w as f32, 20.0], edit);
                            } else {
                                ui.add(edit);
                            }
                        } else {
                            ui.add(edit);
                        };
                    }
                });
                return Deger::Metin(text);
            }
            Deger::Bos
        }),
    );

    globals.insert(
        "büyük_girdi_alanı".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if let Some(Deger::Metin(mut text)) = args.first().cloned() {
                CURRENT_UI.with(|c| {
                    if let Some(ui_ptr) = *c.borrow() {
                        let ui = unsafe { &mut *ui_ptr };
                        ui.add(egui::TextEdit::multiline(&mut text));
                    }
                });
                return Deger::Metin(text);
            }
            Deger::Bos
        }),
    );

    globals.insert(
        "kaydırıcı".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if args.len() >= 3 {
                if let (Deger::Sayi(mut val), Deger::Sayi(min), Deger::Sayi(max)) =
                    (&args[0], &args[1], &args[2])
                {
                    CURRENT_UI.with(|c| {
                        if let Some(ui_ptr) = *c.borrow() {
                            let ui = unsafe { &mut *ui_ptr };
                            ui.add(egui::Slider::new(&mut val, *min..=*max));
                        }
                    });
                    return Deger::Sayi(val);
                }
            }
            args.first().cloned().unwrap_or(Deger::Sayi(0.0))
        }),
    );

    globals.insert(
        "onay_kutusu".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if args.len() >= 2 {
                if let (Deger::Sayi(durum), Deger::Metin(metin)) = (&args[0], &args[1]) {
                    let mut is_checked = *durum != 0.0;
                    CURRENT_UI.with(|c| {
                        if let Some(ui_ptr) = *c.borrow() {
                            let ui = unsafe { &mut *ui_ptr };
                            ui.checkbox(&mut is_checked, metin);
                        }
                    });
                    return Deger::Sayi(if is_checked { 1.0 } else { 0.0 });
                }
            }
            args.first().cloned().unwrap_or(Deger::Sayi(0.0))
        }),
    );

    globals.insert(
        "yan_yana".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if let Some(fks) = args.first() {
                CURRENT_UI.with(|c| {
                    let ui_ptr_opt = *c.borrow();
                    if let Some(ui_ptr) = ui_ptr_opt {
                        let outer_ui = unsafe { &mut *ui_ptr };
                        outer_ui.horizontal(|ui| {
                            let inner_ui_ptr = ui as *mut egui::Ui;
                            *c.borrow_mut() = Some(inner_ui_ptr);

                            CURRENT_INTERP.with(|i| {
                                let interp_ptr_opt = *i.borrow();
                                if let Some(interp_ptr) = interp_ptr_opt {
                                    let interp = unsafe { &mut *interp_ptr };
                                    interp.fonksiyon_cagrisi(fks.clone(), vec![]);
                                }
                            });
                        });
                        // Restore outer ui
                        *c.borrow_mut() = Some(ui_ptr);
                    }
                });
            }
            Deger::Bos
        }),
    );

    globals.insert(
        "ayraç".to_string(),
        Deger::DahiliFonksiyon(|_args| {
            CURRENT_UI.with(|c| {
                if let Some(ui_ptr) = *c.borrow() {
                    let ui = unsafe { &mut *ui_ptr };
                    ui.separator();
                }
            });
            Deger::Bos
        }),
    );

    globals.insert(
        "boşluk".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if let Some(Deger::Sayi(miktar)) = args.first() {
                CURRENT_UI.with(|c| {
                    if let Some(ui_ptr) = *c.borrow() {
                        let ui = unsafe { &mut *ui_ptr };
                        ui.add_space(*miktar as f32);
                    }
                });
            }
            Deger::Bos
        }),
    );

    // Gelişmiş GUI Bileşenleri (Sürüm 0.2.0)

    globals.insert(
        "sekme".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if args.len() >= 2 {
                if let (Deger::Sayi(secili_mi), Deger::Metin(metin)) = (&args[0], &args[1]) {
                    let is_selected = *secili_mi != 0.0;
                    let mut clicked = false;
                    CURRENT_UI.with(|c| {
                        if let Some(ui_ptr) = *c.borrow() {
                            let ui = unsafe { &mut *ui_ptr };
                            if ui.selectable_label(is_selected, metin).clicked() {
                                clicked = true;
                            }
                        }
                    });
                    return Deger::Sayi(if clicked { 1.0 } else { 0.0 });
                }
            }
            Deger::Sayi(0.0)
        }),
    );

    globals.insert(
        "yüzen_pencere".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if args.len() >= 3 {
                if let (Deger::Metin(baslik), Deger::Sayi(acik_mi), fks) =
                    (&args[0], &args[1], &args[2])
                {
                    let mut is_open = *acik_mi != 0.0;
                    CURRENT_UI.with(|c| {
                        let ui_ptr_opt = *c.borrow();
                        if let Some(ui_ptr) = ui_ptr_opt {
                            let outer_ui = unsafe { &mut *ui_ptr };
                            let ctx = outer_ui.ctx().clone();

                            egui::Window::new(baslik)
                                .open(&mut is_open)
                                .show(&ctx, |ui| {
                                    let inner_ui_ptr = ui as *mut egui::Ui;
                                    *c.borrow_mut() = Some(inner_ui_ptr);

                                    CURRENT_INTERP.with(|i| {
                                        let interp_ptr_opt = *i.borrow();
                                        if let Some(interp_ptr) = interp_ptr_opt {
                                            let interp = unsafe { &mut *interp_ptr };
                                            interp.fonksiyon_cagrisi(fks.clone(), vec![]);
                                        }
                                    });
                                });
                            // Restore outer
                            *c.borrow_mut() = Some(ui_ptr);
                        }
                    });
                    return Deger::Sayi(if is_open { 1.0 } else { 0.0 });
                }
            }
            Deger::Sayi(1.0)
        }),
    );

    globals.insert(
        "menü_çubuğu".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if let Some(fks) = args.first() {
                CURRENT_UI.with(|c| {
                    let ui_ptr_opt = *c.borrow();
                    if let Some(ui_ptr) = ui_ptr_opt {
                        let outer_ui = unsafe { &mut *ui_ptr };
                        egui::menu::bar(outer_ui, |ui| {
                            let inner_ui_ptr = ui as *mut egui::Ui;
                            *c.borrow_mut() = Some(inner_ui_ptr);

                            CURRENT_INTERP.with(|i| {
                                let interp_ptr_opt = *i.borrow();
                                if let Some(interp_ptr) = interp_ptr_opt {
                                    let interp = unsafe { &mut *interp_ptr };
                                    interp.fonksiyon_cagrisi(fks.clone(), vec![]);
                                }
                            });
                        });
                        *c.borrow_mut() = Some(ui_ptr);
                    }
                });
            }
            Deger::Bos
        }),
    );

    globals.insert(
        "açılır_menü".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if args.len() >= 2 {
                if let (Deger::Metin(baslik), fks) = (&args[0], &args[1]) {
                    CURRENT_UI.with(|c| {
                        let ui_ptr_opt = *c.borrow();
                        if let Some(ui_ptr) = ui_ptr_opt {
                            let outer_ui = unsafe { &mut *ui_ptr };
                            outer_ui.menu_button(baslik, |ui| {
                                let inner_ui_ptr = ui as *mut egui::Ui;
                                *c.borrow_mut() = Some(inner_ui_ptr);

                                CURRENT_INTERP.with(|i| {
                                    let interp_ptr_opt = *i.borrow();
                                    if let Some(interp_ptr) = interp_ptr_opt {
                                        let interp = unsafe { &mut *interp_ptr };
                                        interp.fonksiyon_cagrisi(fks.clone(), vec![]);
                                    }
                                });
                            });
                            *c.borrow_mut() = Some(ui_ptr);
                        }
                    });
                }
            }
            Deger::Bos
        }),
    );

    globals.insert(
        "grup_kutusu".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if args.len() >= 2 {
                if let (Deger::Metin(baslik), fks) = (&args[0], &args[1]) {
                    CURRENT_UI.with(|c| {
                        let ui_ptr_opt = *c.borrow();
                        if let Some(ui_ptr) = ui_ptr_opt {
                            let outer_ui = unsafe { &mut *ui_ptr };
                            outer_ui.group(|ui| {
                                ui.label(baslik.clone());
                                ui.separator();
                                let inner_ui_ptr = ui as *mut egui::Ui;
                                *c.borrow_mut() = Some(inner_ui_ptr);

                                CURRENT_INTERP.with(|i| {
                                    let interp_ptr_opt = *i.borrow();
                                    if let Some(interp_ptr) = interp_ptr_opt {
                                        let interp = unsafe { &mut *interp_ptr };
                                        interp.fonksiyon_cagrisi(fks.clone(), vec![]);
                                    }
                                });
                            });
                            *c.borrow_mut() = Some(ui_ptr);
                        }
                    });
                }
            }
            Deger::Bos
        }),
    );

    globals.insert(
        "alt_alta".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if let Some(fks) = args.first() {
                CURRENT_UI.with(|c| {
                    let ui_ptr_opt = *c.borrow();
                    if let Some(ui_ptr) = ui_ptr_opt {
                        let outer_ui = unsafe { &mut *ui_ptr };
                        outer_ui.vertical(|ui| {
                            let inner_ui_ptr = ui as *mut egui::Ui;
                            *c.borrow_mut() = Some(inner_ui_ptr);

                            CURRENT_INTERP.with(|i| {
                                let interp_ptr_opt = *i.borrow();
                                if let Some(interp_ptr) = interp_ptr_opt {
                                    let interp = unsafe { &mut *interp_ptr };
                                    interp.fonksiyon_cagrisi(fks.clone(), vec![]);
                                }
                            });
                        });
                        *c.borrow_mut() = Some(ui_ptr);
                    }
                });
            }
            Deger::Bos
        }),
    );

    globals.insert(
        "grid_oluştur".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if args.len() >= 2 {
                if let (Deger::Metin(id), fks) = (&args[0], &args[1]) {
                    CURRENT_UI.with(|c| {
                        let ui_ptr_opt = *c.borrow();
                        if let Some(ui_ptr) = ui_ptr_opt {
                            let outer_ui = unsafe { &mut *ui_ptr };
                            egui::Grid::new(id).striped(true).show(outer_ui, |ui| {
                                let inner_ui_ptr = ui as *mut egui::Ui;
                                *c.borrow_mut() = Some(inner_ui_ptr);

                                CURRENT_INTERP.with(|i| {
                                    let interp_ptr_opt = *i.borrow();
                                    if let Some(interp_ptr) = interp_ptr_opt {
                                        let interp = unsafe { &mut *interp_ptr };
                                        interp.fonksiyon_cagrisi(fks.clone(), vec![]);
                                    }
                                });
                            });
                            *c.borrow_mut() = Some(ui_ptr);
                        }
                    });
                }
            }
            Deger::Bos
        }),
    );

    globals.insert(
        "satır_bitir".to_string(),
        Deger::DahiliFonksiyon(|_args| {
            CURRENT_UI.with(|c| {
                if let Some(ui_ptr) = *c.borrow() {
                    let ui = unsafe { &mut *ui_ptr };
                    ui.end_row();
                }
            });
            Deger::Bos
        }),
    );

    globals.insert(
        "kaydırılabilir_alan".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if args.len() >= 2 {
                if let (Deger::Metin(id), fks) = (&args[0], &args[1]) {
                    CURRENT_UI.with(|c| {
                        let ui_ptr_opt = *c.borrow();
                        if let Some(ui_ptr) = ui_ptr_opt {
                            let outer_ui = unsafe { &mut *ui_ptr };
                            egui::ScrollArea::vertical()
                                .id_source(id)
                                .auto_shrink([false, false])
                                .show(outer_ui, |ui| {
                                    let inner_ui_ptr = ui as *mut egui::Ui;
                                    *c.borrow_mut() = Some(inner_ui_ptr);

                                    CURRENT_INTERP.with(|i| {
                                        let interp_ptr_opt = *i.borrow();
                                        if let Some(interp_ptr) = interp_ptr_opt {
                                            let interp = unsafe { &mut *interp_ptr };
                                            interp.fonksiyon_cagrisi(fks.clone(), vec![]);
                                        }
                                    });
                                });
                            *c.borrow_mut() = Some(ui_ptr);
                        }
                    });
                }
            }
            Deger::Bos
        }),
    );

    globals.insert(
        "alan_ayır".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if args.len() >= 3 {
                if let (Deger::Sayi(w), Deger::Sayi(h), fks) = (&args[0], &args[1], &args[2]) {
                    CURRENT_UI.with(|c| {
                        let ui_ptr_opt = *c.borrow();
                        if let Some(ui_ptr) = ui_ptr_opt {
                            let outer_ui = unsafe { &mut *ui_ptr };
                            outer_ui.allocate_ui(egui::Vec2::new(*w as f32, *h as f32), |ui| {
                                let inner_ui_ptr = ui as *mut egui::Ui;
                                *c.borrow_mut() = Some(inner_ui_ptr);

                                CURRENT_INTERP.with(|i| {
                                    let interp_ptr_opt = *i.borrow();
                                    if let Some(interp_ptr) = interp_ptr_opt {
                                        let interp = unsafe { &mut *interp_ptr };
                                        interp.fonksiyon_cagrisi(fks.clone(), vec![]);
                                    }
                                });
                            });
                            *c.borrow_mut() = Some(ui_ptr);
                        }
                    });
                }
            }
            Deger::Bos
        }),
    );

    globals.insert(
        "boyutlu_buton".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if args.len() >= 3 {
                if let (Deger::Metin(metin), Deger::Sayi(w), Deger::Sayi(h)) =
                    (&args[0], &args[1], &args[2])
                {
                    let mut clicked = false;
                    CURRENT_UI.with(|c| {
                        if let Some(ui_ptr) = *c.borrow() {
                            let ui = unsafe { &mut *ui_ptr };
                            if ui
                                .add_sized([*w as f32, *h as f32], egui::Button::new(metin))
                                .clicked()
                            {
                                clicked = true;
                            }
                        }
                    });
                    return Deger::Sayi(if clicked { 1.0 } else { 0.0 });
                }
            }
            Deger::Sayi(0.0)
        }),
    );

    globals.insert(
        "boyutlu_renkli_buton".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if args.len() >= 6 {
                if let (
                    Deger::Metin(metin),
                    Deger::Sayi(w),
                    Deger::Sayi(h),
                    Deger::Sayi(r),
                    Deger::Sayi(g),
                    Deger::Sayi(b),
                ) = (&args[0], &args[1], &args[2], &args[3], &args[4], &args[5])
                {
                    let mut clicked = false;
                    CURRENT_UI.with(|c| {
                        if let Some(ui_ptr) = *c.borrow() {
                            let ui = unsafe { &mut *ui_ptr };
                            let btn = egui::Button::new(
                                egui::RichText::new(metin.clone())
                                    .color(egui::Color32::from_rgb(*r as u8, *g as u8, *b as u8)),
                            );
                            if ui.add_sized([*w as f32, *h as f32], btn).clicked() {
                                clicked = true;
                            }
                        }
                    });
                    return Deger::Sayi(if clicked { 1.0 } else { 0.0 });
                }
            }
            Deger::Sayi(0.0)
        }),
    );

    globals.insert(
        "etiket".to_string(),
        Deger::DahiliFonksiyon(|args| {
            if let Some(metin) = args.first() {
                CURRENT_UI.with(|c| {
                    if let Some(ui_ptr) = *c.borrow() {
                        let ui = unsafe { &mut *ui_ptr };

                        let mut rich = egui::RichText::new(metin.to_string());
                        let mut is_heading = false;

                        // etiket(metin)
                        // etiket(metin, r, g, b)
                        // etiket(metin, stil_str) -> "kalın", "eğik", "başlık"

                        if args.len() >= 2 {
                            if let Deger::Metin(stil) = &args[1] {
                                match stil.as_str() {
                                    "kalın" => rich = rich.strong(),
                                    "eğik" => rich = rich.italics(),
                                    "başlık" => is_heading = true,
                                    _ => {}
                                }
                            } else if args.len() >= 4 {
                                // Renk
                                if let (Deger::Sayi(r), Deger::Sayi(g), Deger::Sayi(b)) =
                                    (&args[1], &args[2], &args[3])
                                {
                                    rich = rich.color(egui::Color32::from_rgb(
                                        *r as u8, *g as u8, *b as u8,
                                    ));
                                }
                            }
                        }

                        if is_heading {
                            ui.heading(rich);
                        } else {
                            ui.label(rich);
                        }
                    }
                });
            }
            Deger::Bos
        }),
    );
}

pub fn gui_istegi_var_mi() -> bool {
    GUI_REQUEST.with(|r| r.borrow().is_some())
}

pub fn gui_calistir(interp: Yorumlayici) {
    let req = GUI_REQUEST.with(|r| r.borrow_mut().take());

    if let Some(req) = req {
        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([req.genislik, req.yukseklik]),
            ..Default::default()
        };
        let baslik_copy = req.baslik.clone();
        let _ = eframe::run_native(
            &baslik_copy,
            options,
            Box::new(|_cc| Box::new(HumaGuiApp::new(req.baslik, req.cizim_fks, interp))),
        );
    }
}
