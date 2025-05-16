mod flamegraph;
mod helpers;
mod overview;

use crate::ui::flamegraph::{draw_flamegraph, Options, StackFrame};
use common::parser::AccumulatedData;
use eframe::emath::Align;
use egui::Layout;

pub fn run_ui(data: MemInfo) -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1024.0, 720.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Memgraph",
        options,
        Box::new(|cc| Ok(Box::new(MemgraphApp::new(data)))),
    )
}

#[derive(PartialEq, Debug)]
enum MainTab {
    Overview,
    Charts,
    Flamegraph,
}

pub struct MemInfo {
    pub app_name: String,
    pub data: AccumulatedData,
}

struct MemgraphApp {
    info: MemInfo,
    current_tab: MainTab,
}

impl MemgraphApp {
    pub fn new(info: MemInfo) -> Self {
        Self {
            info,
            current_tab: MainTab::Flamegraph,
        }
    }
}

impl eframe::App for MemgraphApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.with_layout(Layout::top_down_justified(Align::Center), |ui| {
                ui.horizontal(|ui| {
                    for tab in [MainTab::Overview, MainTab::Charts, MainTab::Flamegraph] {
                        let selected = self.current_tab == tab;
                        if ui
                            .selectable_label(selected, format!("{:?}", tab))
                            .clicked()
                        {
                            self.current_tab = tab;
                        }
                    }
                });

                ui.separator();

                match self.current_tab {
                    MainTab::Overview => {
                        overview::show(ui, &self.info);
                    }
                    MainTab::Charts => {
                        ui.label("This is the Charts tab.");
                    }
                    MainTab::Flamegraph => {
                        let options = Options { frame_height: 20.0 };

                        let root = [
                            StackFrame {
                                label: "main".to_string(),
                                value: 5.0,
                                children: vec![
                                    StackFrame {
                                        label: "ch1".to_string(),
                                        value: 3.0,
                                        children: vec![],
                                    },
                                    StackFrame {
                                        label: "ch2".to_string(),
                                        value: 2.0,
                                        children: vec![],
                                    },
                                ],
                            },
                            StackFrame {
                                label: "side".to_string(),
                                value: 7.0,
                                children: vec![],
                            },
                        ];

                        draw_flamegraph(ui, options, &root);
                    }
                }
            });
        });
    }
}
