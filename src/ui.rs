use common::parser::AccumulatedData;

pub fn run_ui(data: AccumulatedData) -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([800.0, 600.0]),
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

struct MemgraphApp {
    data: AccumulatedData,
    current_tab: MainTab,
}

impl MemgraphApp {
    pub fn new(data: AccumulatedData) -> Self {
        Self {
            data,
            current_tab: MainTab::Overview,
        }
    }
}

impl eframe::App for MemgraphApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
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
                    ui.label("This is the Overview tab.");
                }
                MainTab::Charts => {
                    ui.label("This is the Charts tab.");
                }
                MainTab::Flamegraph => {
                    ui.label("This is the Flamegraph tab.");
                }
            }
        });
    }
}
