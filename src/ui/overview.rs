use crate::ui::helpers::add_key_value;
use crate::ui::MemInfo;
use bytesize::ByteSize;
use common::parser::Frame;
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
                let contributes = info.data.allocations.iter().map(|alloc| {
                    let trace = &info.data.traces[(alloc.trace_idx - 1) as usize];
                    let ip = &info.data.instruction_pointers[(trace.ip_idx - 1) as usize];
                    let fn_idx = match ip.frame {
                        Frame::Single { function_idx } => function_idx,
                        Frame::Multiple { function_idx, .. } => function_idx,
                    };
                    let fn_name = &info.data.strings[fn_idx];
                    [fn_name.to_string(), alloc.data.peak.to_string()]
                });

                add_table(
                    col1,
                    "Peak Contributions",
                    ["Location", "Peak"],
                    contributes,
                );
            });
            ui.add_space(20.0);
        })
    });
}

fn add_table<'a, const N: usize>(
    ui: &mut Ui,
    label: &str,
    headers: [&'a str; N],
    data: impl IntoIterator<Item = [impl ToString; N]>,
) {
    ui.with_layout(Layout::top_down_justified(Align::Center), |ui| {
        ui.label(label);
        TableBuilder::new(ui)
            .striped(true)
            .resizable(true)
            .cell_layout(Layout::left_to_right(Align::Center))
            .column(Column::remainder().clip(true))
            .column(Column::remainder())
            .header(20.0, |mut header| {
                for header_label in headers {
                    header.col(|ui| {
                        ui.label(header_label);
                    });
                }
            })
            .body(|mut body| {
                for a in data {
                    body.row(20.0, |mut row| {
                        row.col(|ui| {
                            ui.label(a[0].to_string());
                        });
                        row.col(|ui| {
                            ui.label(a[1].to_string());
                        });
                    })
                }
            });
    });
}
