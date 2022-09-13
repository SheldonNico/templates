use std::{io::Read, path::Path};
use eframe::{egui, glow::{self, HasContext}};
use std::fmt::Write;

fn main() {
    let options = eframe::NativeOptions {
        decorated: true,
        transparent: true,
        // min_window_size: Some(egui::vec2(320.0, 100.0)),
        ..Default::default()
    };
    eframe::run_native(
        "Demo",
        options,
        Box::new(|cc| Box::new(App::new(cc)))
    );
}

fn setup_custom_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    fonts.font_data.insert(
        "Cascadia Code".to_owned(),
        egui::FontData::from_owned(read_bytes("assets/Cascadia_Code/CascadiaCode.ttf"))
    );
    fonts.families.entry(egui::FontFamily::Proportional).or_default().insert(0, "Cascadia Code".to_owned());
    fonts.families.entry(egui::FontFamily::Monospace).or_default().insert(0, "Cascadia Code".to_owned());

    ctx.set_fonts(fonts);
}

fn setup_custom_style(ctx: &egui::Context) {
    use egui::{TextStyle, FontId, FontFamily::Proportional};
    let mut style = (*ctx.style()).clone();
    style.text_styles = [
        (TextStyle::Heading, FontId::new(30.0, Proportional)),
        (egui::TextStyle::Name("Heading2".into()), FontId::new(25.0, Proportional)),
        (TextStyle::Body, FontId::new(18.0, Proportional)),
        (TextStyle::Monospace, FontId::new(14.0, Proportional)),
        (TextStyle::Button, FontId::new(14.0, Proportional)),
        (TextStyle::Small, FontId::new(10.0, Proportional)),
    ].into();
    ctx.set_style(style);
}

impl App {
    pub fn new(cc: &eframe::CreationContext) -> Self {
        setup_custom_fonts(&cc.egui_ctx);
        setup_custom_style(&cc.egui_ctx);

        Self {
            name: "xi.nie".to_owned(),
            age: 42,

            svg_image: egui_extras::RetainedImage::from_svg_bytes("demo.svg", &read_bytes("assets/demo.svg")).unwrap(),
            picked_path: None,
            dropped_files: vec![],
            take_screenshot: false,
            screenshot: None,
            texture: None,

            exit_confirmed: false,
            exit_confirmation: false,
        }
    }
}

struct App {
    name: String,
    age: u32,

    svg_image: egui_extras::RetainedImage,
    picked_path: Option<String>,
    dropped_files: Vec<egui::DroppedFile>,

    take_screenshot: bool,
    screenshot: Option<egui::ColorImage>,
    texture: Option<egui::TextureHandle>,

    exit_confirmation: bool,
    exit_confirmed: bool,
}

impl eframe::App for App {
    fn on_close_event(&mut self) -> bool {
        self.exit_confirmation = true;
        self.exit_confirmed
    }

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        use egui::*;
        use plot::{Plot, Legend, Line, PlotPoints};


        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("My egui Application");
            ui.horizontal(|ui| {
                ui.label("Your name:");
                ui.text_edit_singleline(&mut self.name)
            });

            ui.add(egui::Slider::new(&mut self.age, 0..=120).text("age"));
            ui.horizontal(|ui| {
                if ui.button("Increase").clicked() {
                    self.age = self.age.saturating_add(1).clamp(0, 120);
                }

                if ui.button("Decrease").clicked() {
                    self.age = self.age.saturating_sub(1).clamp(0, 120);
                }
            });
            ui.label(format!("Hello `{}`, age {}", self.name, self.age));

            if ui.button("Open file…").clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_file() {
                    self.picked_path = Some(path.display().to_string());
                }
            }

            if let Some(picked_path) = &self.picked_path {
                ui.horizontal(|ui| {
                    ui.label("Picked file:");
                    ui.monospace(picked_path);
                });
            }

            if !self.dropped_files.is_empty() {
                ui.group(|ui| {
                    ui.label("Dropped files:");

                    for file in &self.dropped_files {
                        let mut info = if let Some(path) = &file.path {
                            path.display().to_string()
                        } else if !file.name.is_empty() {
                            file.name.clone()
                        } else {
                            "???".to_owned()
                        };

                        if let Some(bytes) = &file.bytes {
                            write!(info, " ({} bytes)", bytes.len()).ok();
                        }
                        ui.label(info);
                    }
                });
            }

            ui.separator();
            ui.add_space(5.);
            ui.label(egui::RichText::new("Sub Heading").text_style(egui::TextStyle::Name("Heading2".into())).strong());
            ui.label(LOREM_IPSUM);
            ui.add_space(15.);

            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("take screenshot").clicked() {
                    self.take_screenshot = true;
                }
                if ui.button("clear screenshot").clicked() {
                    self.texture = None;
                }
            });

            if let Some(screenshot) = self.screenshot.take() {
                self.texture = Some(ui.ctx().load_texture(
                    "screenshot", screenshot, egui::TextureFilter::Linear
                ));
            }

            ui.separator();

            let mut lines = Plot::new("lines").legend(Legend::default());
            let data = Line::new(PlotPoints::from_explicit_callback(
                move |x| 0.5 * (2.0*x).sin(),
                ..,
                512
            ))
            .color(Color32::from_rgb(200, 100, 100))
            .name("wave");

            lines.show(ui, |plot_ui| {
                plot_ui.line(data)
            });


            // if let Some(texture) = &self.texture {
            //     ui.image(texture, ui.available_size());
            // } else {
            //     ui.spinner();
            // }

            // ui.separator();
            // self.svg_image.show_max_size(ui, ui.available_size());
        });

        if !ctx.input().raw.hovered_files.is_empty() {
            let mut text = "Dropping files:\n".to_owned();

            for file in &ctx.input().raw.hovered_files {
                if let Some(path) = &file.path {
                    write!(text, "\n{}", path.display()).ok();
                } else if !file.mime.is_empty() {
                    write!(text, "\n{}", file.mime).ok();
                } else {
                    text += "\n???";
                }
            }

            let painter = ctx.layer_painter(LayerId::new(Order::Foreground, Id::new("file drop target")));

            let screen_rect = ctx.input().screen_rect();
            painter.rect_filled(screen_rect, 0.0, Color32::from_black_alpha(192));
            painter.text(
                screen_rect.center(),
                Align2::CENTER_CENTER,
                text,
                TextStyle::Heading.resolve(&ctx.style()),
                Color32::WHITE,
            );
        }

        if !ctx.input().raw.dropped_files.is_empty() {
            self.dropped_files = ctx.input().raw.dropped_files.clone();
        }

        if self.exit_confirmation {
            egui::Window::new("Do you want to quit?")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        if ui.button(egui::RichText::from("Yes").strong()).clicked() {
                            self.exit_confirmed = true;
                            frame.close();
                        }

                        if ui.button("No").clicked() {
                            self.exit_confirmation = false;
                        }
                    });
                });
        }
    }

    fn post_rendering(&mut self, screen_size_px: [u32; 2], frame: &eframe::Frame) {
        if !self.take_screenshot { return ;}
        self.take_screenshot = false;

        if let Some(gl) = frame.gl() {
            let mut buf = vec![0u8; screen_size_px[0] as usize * screen_size_px[1] as usize * 4];
            let pixels = glow::PixelPackData::Slice(&mut buf[..]);

            unsafe {
                gl.read_pixels(0, 0, screen_size_px[0] as i32, screen_size_px[1] as i32, glow::RGBA, glow::UNSIGNED_BYTE, pixels);
            }

            let mut rows: Vec<Vec<u8>> = buf
                .chunks(screen_size_px[0] as usize * 4)
                .into_iter()
                .map(|chunk| chunk.iter().map(Clone::clone).collect())
                .collect();
            rows.reverse();
            let buf: Vec<u8> = rows.into_iter().flatten().collect();
            self.screenshot = Some(egui::ColorImage::from_rgba_unmultiplied(
                [screen_size_px[0] as usize, screen_size_px[1] as usize], &buf[..]
            ));
        }
    }
}

fn read_bytes(filename: impl AsRef<Path>) -> Vec<u8> {
    let msg = format!("fail to read bytes from file: {}", filename.as_ref().display());
    let fh = std::fs::File::open(filename).expect(&msg);
    Vec::from_iter(fh.bytes().map(|b| b.expect(&msg)))
}


pub const LOREM_IPSUM: &str = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.";

