use crate::ui::helpers::add_key_value;
use crate::ui::MemInfo;
use bytesize::ByteSize;
use eframe::emath::Align;
use egui::{Layout, Ui};
use egui_extras::{Column, TableBuilder};

pub fn show(ui: &mut Ui, info: &MemInfo) {
    ui.with_layout(Layout::default(), |ui| {
        ui.add_space(20.0);
        ui.horizontal(|ui| {
            ui.add_space(20.0);
            ui.columns(3, |columns| {
                let [col1, col2, col3] = columns.get_disjoint_mut([0, 1, 2]).unwrap();

                let total_ram = info.data.page_size * info.data.pages;

                add_key_value(col1, "application", &info.app_name);
                add_key_value(col1, "total runtime", format!("{:?}", info.data.duration));
                add_key_value(col1, "total system memory", ByteSize::b(total_ram));

                add_key_value(
                    col2,
                    "calls to allocation functions",
                    info.data.total.allocations,
                );
                add_key_value(col2, "temporary allocations", info.data.total.temporary);

                add_key_value(
                    col3,
                    "peak heap memory consumption",
                    ByteSize::b(info.data.total.peak),
                );
                add_key_value(col3, "peak RSS", ByteSize::b(info.data.peak_rss));
                add_key_value(col3, "total memory leaked", info.data.total.leaked);
            });
            ui.add_space(20.0);
        });
        ui.add_space(20.0);
        ui.separator();
        ui.add_space(20.0);
        ui.horizontal(|ui| {
            ui.add_space(20.0);
            ui.columns(4, |columns| {
                let [col1, col2, col3, col4] = columns.get_disjoint_mut([0, 1, 2, 3]).unwrap();
                col1.with_layout(Layout::top_down_justified(Align::Center), |ui| {
                    ui.label("Peak Contributions");
                    TableBuilder::new(ui)
                        .striped(true)
                        .resizable(true)
                        .cell_layout(Layout::left_to_right(Align::Center))
                        .column(Column::remainder())
                        .column(Column::remainder())
                        .header(20.0, |mut header| {
                            header.col(|ui| {
                                ui.label("Location");
                            });
                            header.col(|ui| {
                                ui.label("Peak");
                            });
                        })
                        .body(|mut body| {
                            body.row(20.0, |mut row| {
                                row.col(|ui| {
                                    ui.label("Key");
                                });
                                row.col(|ui| {
                                    ui.label("Value");
                                });
                            })
                        });
                });
            });
            ui.add_space(20.0);
        })
    });
}

fn expanding_content(ui: &mut Ui) {
    ui.add(egui::Separator::default().vertical());
}
