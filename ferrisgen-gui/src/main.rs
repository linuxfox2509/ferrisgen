#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

use eframe::{egui, NativeOptions};
use ferrisgen::{generate, Options, estimate_entropy};

fn main() -> eframe::Result<()> {
    let mut native = NativeOptions::default();
    native.initial_window_size = Some(egui::vec2(760.0, 420.0));
    eframe::run_native("FerrisGen", native, Box::new(|_cc| Box::new(AppState::default())))
}

struct AppState {
    length: u8,
    lowercase: bool,
    uppercase: bool,
    numbers: bool,
    special: bool,
    no_ambiguous: bool,

    auto_copy: bool,
    show_password: bool,
    output: String,
    message: String,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            length: 12,
            lowercase: true,
            uppercase: true,
            numbers: true,
            special: false,
            no_ambiguous: false,
            auto_copy: false,
            show_password: false,
            output: String::new(),
            message: String::new(),
        }
    }
}

impl eframe::App for AppState {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(6.0);
//            ui.vertical_centered(|ui| {
//                ui.heading(egui::RichText::new("FerrisGen").heading());
//                ui.add_space(6.0);
//            });

            egui::Frame::group(&ui.style()).show(ui, |ui| {
                ui.columns(2, |columns| {
                    let left = &mut columns[0];
                    left.vertical(|ui| {
                        ui.label(egui::RichText::new("Options").strong());
                        ui.add_space(6.0);
                        ui.horizontal(|ui| {
                            ui.label("Length:");
                            ui.add(egui::Slider::new(&mut self.length, 6..=25).clamp_to_range(true));
                            ui.label(egui::RichText::new(format!("{}", self.length)).strong());
                        });

                        ui.add_space(6.0);
                        ui.horizontal(|ui| {
                            ui.checkbox(&mut self.lowercase, "Lowercase");
                            ui.checkbox(&mut self.uppercase, "Uppercase");
                        });
                        ui.horizontal(|ui| {
                            ui.checkbox(&mut self.numbers, "Numbers");
                            ui.checkbox(&mut self.special, "Special");
                        });

                        ui.add_space(6.0);
                        ui.horizontal(|ui| {
                            ui.checkbox(&mut self.no_ambiguous, "Exclude ambiguous");
                            ui.checkbox(&mut self.auto_copy, "Auto-copy");
                        });

                        ui.add_space(8.0);
                        // Generate button big and colored with high-contrast white text
                        let gen_button = egui::Button::new(egui::RichText::new("Generate").color(egui::Color32::WHITE)).min_size(egui::vec2(120.0, 36.0));
                        if ui.add(gen_button.fill(egui::Color32::from_rgb(16, 185, 129))).clicked() {
                            let opts = Options {
                                length: self.length as usize,
                                lowercase: self.lowercase,
                                uppercase: self.uppercase,
                                numbers: self.numbers,
                                special: self.special,
                                no_ambiguous: self.no_ambiguous,
                            };

                            match generate(&opts) {
                                Ok(p) => {
                                    self.output = p.clone();
                                    if self.auto_copy {
                                        match arboard::Clipboard::new().and_then(|mut c| c.set_text(p.clone())) {
                                            Ok(_) => self.message = "Generated and copied to clipboard".to_string(),
                                            Err(e) => self.message = format!("Generated but copy failed: {}", e),
                                        }
                                    } else {
                                        self.message = "Generated".to_string();
                                    }
                                }
                                Err(e) => {
                                    self.message = format!("Error: {}", e);
                                }
                            }
                        }

                        ui.add_space(4.0);
                        if !self.message.is_empty() {
                            ui.label(egui::RichText::new(&self.message).small());
                        }
                    });

                    let right = &mut columns[1];
                    right.vertical(|ui| {
                        ui.label(egui::RichText::new("Password").strong());
                        ui.add_space(6.0);

                        let display = if self.show_password { self.output.clone() } else { "*".repeat(self.output.chars().count()) };
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(display).monospace());
                            if ui.small_button(if self.show_password { "Hide" } else { "Show" }).clicked() {
                                self.show_password = !self.show_password;
                            }
                            if ui.add(egui::Button::new("📋 Copy")).clicked() {
                                if !self.output.is_empty() {
                                    match arboard::Clipboard::new().and_then(|mut c| c.set_text(self.output.clone())) {
                                        Ok(_) => self.message = "Copied to clipboard".to_string(),
                                        Err(e) => self.message = format!("Copy failed: {}", e),
                                    }
                                }
                            }
                        });

                        ui.add_space(6.0);
                        ui.label(egui::RichText::new("Strength").strong());
                        ui.add_space(4.0);

                        let opts_for_calc = Options {
                            length: self.length as usize,
                            lowercase: self.lowercase,
                            uppercase: self.uppercase,
                            numbers: self.numbers,
                            special: self.special,
                            no_ambiguous: self.no_ambiguous,
                        };

                        let entropy = estimate_entropy(&opts_for_calc);
                        let (label, color, frac) = {
                            let frac = (entropy / 128.0).min(1.0).max(0.0);
                            if entropy < 28.0 {
                                ("Very weak", egui::Color32::from_rgb(220, 38, 38), frac)
                            } else if entropy < 36.0 {
                                ("Weak", egui::Color32::from_rgb(234, 88, 12), frac)
                            } else if entropy < 60.0 {
                                ("Fair", egui::Color32::from_rgb(234, 179, 8), frac)
                            } else if entropy < 128.0 {
                                ("Strong", egui::Color32::from_rgb(34, 197, 94), frac)
                            } else {
                                ("Very strong", egui::Color32::from_rgb(16, 185, 129), frac)
                            }
                        };

                        let pb = egui::ProgressBar::new(frac).text(format!("{:.0} bits - {}", entropy, label));
                        ui.add_sized([300.0, 24.0], pb.fill(color));

                        let info = ui.small_button("ℹ");
                        info.on_hover_text("Estimated entropy = length × log2(pool size). Pool size counts selected character classes; ambiguous characters are excluded when 'Exclude ambiguous' is checked. This is an approximation used for guidance only.");

                        ui.add_space(8.0);
                        let mut classes = Vec::new();
                        if self.lowercase { classes.push("lowercase") }
                        if self.uppercase { classes.push("uppercase") }
                        if self.numbers { classes.push("numbers") }
                        if self.special { classes.push("special") }
                        ui.label(egui::RichText::new(format!("Included: {}", classes.join(", "))).small());

                    });
                });
            });

            ui.add_space(6.0);
            ui.horizontal_centered(|ui| {
                ui.label(egui::RichText::new("Tips: use the checkboxes to control included characters and toggle 'Auto-copy' to automatically copy new passwords").small());
            });
        });
    }
}