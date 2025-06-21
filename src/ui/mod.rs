mod flamegraph;
mod helpers;
mod overview;
mod topdown;
mod widgets;

use crate::ui::flamegraph::FlamegraphPage;
use crate::ui::topdown::TopDown;
use eframe::emath::Align;
use egui::Layout;
use memtrace_utils::parser::AccumulatedData;

pub fn run_ui(data: MemInfo) -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1024.0, 720.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Memgraph",
        options,
        Box::new(|_| Ok(Box::new(MemgraphApp::new(data)))),
    )
}

#[derive(PartialEq, Debug)]
enum MainTab {
    Overview,
    TopDown,
    Flamegraph,
}

pub struct MemInfo {
    pub app_name: String,
    pub data: AccumulatedData,
}

struct MemgraphApp {
    info: MemInfo,
    current_tab: MainTab,
    fg_page: FlamegraphPage,
    top_down: TopDown,
}

impl MemgraphApp {
    pub fn new(info: MemInfo) -> Self {
        let fg_page = FlamegraphPage::new(&info);

        Self {
            top_down: TopDown::new(&info),
            info,
            current_tab: MainTab::Overview,
            fg_page,
        }
    }
}

impl eframe::App for MemgraphApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.with_layout(Layout::top_down_justified(Align::Center), |ui| {
                ui.horizontal(|ui| {
                    for tab in [MainTab::Overview, MainTab::TopDown, MainTab::Flamegraph] {
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
                    MainTab::TopDown => {
                        self.top_down.show(ui);
                    }
                    MainTab::Flamegraph => {
                        self.fg_page.show(ui);
                    }
                }
            });
        });
    }
}
