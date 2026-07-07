use chrono::Local;
use eframe::egui;
use std::sync::mpsc::{self, Receiver, Sender};

const FONT_CJK: &[u8] = include_bytes!(
    "../enlisted-stats-viewer/assets/fonts/NotoSansSC-Regular.ttf"
);

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Enlisted Stats Viewer")
            .with_inner_size([360.0, 280.0])
            .with_resizable(false),
        ..Default::default()
    };
    eframe::run_native(
        "Enlisted Stats Viewer",
        options,
        Box::new(|cc| {
            let mut fonts = egui::FontDefinitions::default();
            fonts.font_data.insert(
                "NotoSansCJK".to_owned(),
                egui::FontData::from_static(FONT_CJK),
            );
            fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .push("NotoSansCJK".to_owned());
            cc.egui_ctx.set_fonts(fonts);
            Ok(Box::new(App::default()))
        }),
    )
}

#[derive(PartialEq)]
enum Layout {
    Portrait,
    Landscape,
}

#[derive(PartialEq)]
enum Lang {
    ZhCN,
    ZhTW,
    En,
}

impl Lang {
    fn code(&self) -> &'static str {
        match self {
            Lang::ZhCN => "zh-CN",
            Lang::ZhTW => "zh-TW",
            Lang::En => "en",
        }
    }
}

enum WorkerMsg {
    Done,
    Error(String),
}

struct App {
    player_id: String,
    layout: Layout,
    lang: Lang,
    status: String,
    generating: bool,
    rx: Option<Receiver<WorkerMsg>>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            player_id: String::new(),
            layout: Layout::Landscape,
            lang: Lang::ZhCN,
            status: String::new(),
            generating: false,
            rx: None,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Poll background thread
        if let Some(rx) = &self.rx {
            if let Ok(msg) = rx.try_recv() {
                self.generating = false;
                self.rx = None;
                match msg {
                    WorkerMsg::Done => self.status = "生成成功".to_string(),
                    WorkerMsg::Error(e) => self.status = format!("错误: {}", e),
                }
            }
            ctx.request_repaint();
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(8.0);

            // Player ID
            ui.horizontal(|ui| {
                ui.label("玩家 ID");
                ui.add_sized(
                    [220.0, 20.0],
                    egui::TextEdit::singleline(&mut self.player_id)
                        .hint_text("enlistedrollcall.com 上的 ID"),
                );
            });

            ui.add_space(8.0);

            // Layout
            ui.horizontal(|ui| {
                ui.label("风格");
                ui.radio_value(&mut self.layout, Layout::Landscape, "桌面端");
                ui.radio_value(&mut self.layout, Layout::Portrait, "移动端");
            });

            ui.add_space(8.0);

            // Language
            ui.horizontal(|ui| {
                ui.label("语言");
                egui::ComboBox::from_id_salt("lang")
                    .selected_text(match self.lang {
                        Lang::ZhCN => "简体中文",
                        Lang::ZhTW => "繁體中文",
                        Lang::En => "English",
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.lang, Lang::ZhCN, "简体中文");
                        ui.selectable_value(&mut self.lang, Lang::ZhTW, "繁體中文");
                        ui.selectable_value(&mut self.lang, Lang::En, "English");
                    });
            });

            ui.add_space(16.0);

            // Buttons
            ui.horizontal(|ui| {
                let can_generate = !self.generating && !self.player_id.trim().is_empty();

                if ui
                    .add_enabled(can_generate, egui::Button::new("生成"))
                    .clicked()
                {
                    self.on_generate(ctx.clone());
                }

                if ui.button("退出").clicked() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            });

            ui.add_space(8.0);

            if self.generating {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label("正在获取数据并生成图片…");
                });
            } else if !self.status.is_empty() {
                ui.label(&self.status);
            }

            ui.with_layout(egui::Layout::bottom_up(egui::Align::RIGHT), |ui| {
                ui.hyperlink_to(
                    egui::RichText::new("github.com/GuitaristRin/enlisted-stats-desktop")
                        .small()
                        .weak(),
                    "https://github.com/GuitaristRin/enlisted-stats-desktop",
                );
            });
        });
    }
}

impl App {
    fn on_generate(&mut self, ctx: egui::Context) {
        let timestamp = Local::now().format("%Y-%m-%d_%H-%M-%S");
        let default_name = format!("enlisted_stats_{}.png", timestamp);

        let path = rfd::FileDialog::new()
            .set_file_name(&default_name)
            .add_filter("PNG Image", &["png"])
            .save_file();

        let Some(path) = path else {
            return;
        };

        let id = self.player_id.trim().to_string();
        let layout = match self.layout {
            Layout::Landscape => "landscape",
            Layout::Portrait => "portrait",
        };
        let lang = self.lang.code();

        let (tx, rx): (Sender<WorkerMsg>, Receiver<WorkerMsg>) = mpsc::channel();
        self.rx = Some(rx);
        self.generating = true;
        self.status.clear();

        std::thread::spawn(move || {
            let result = enlisted_stat::generate_card(
                &id,
                lang,
                layout,
                path.to_str().unwrap_or("output.png"),
                2.0,
            );
            let msg = match result {
                Ok(_) => WorkerMsg::Done,
                Err(e) => WorkerMsg::Error(e.to_string()),
            };
            let _ = tx.send(msg);
            ctx.request_repaint();
        });
    }
}
