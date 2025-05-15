use crate::ui::helpers::add_key_value;
use crate::ui::MemInfo;
use bytesize::ByteSize;
use common::parser::{AccumulatedData, Frame};
use eframe::emath::Align;
use egui::{Layout, Ui};
use egui_extras::{Column, TableBuilder};
use itertools::Itertools;
use std::time::Instant;

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
                add_key_value(
                    col3,
                    "total memory leaked",
                    ByteSize::b(info.data.total.leaked),
                );
            });
            ui.add_space(20.0);
        });
        ui.add_space(20.0);
        ui.separator();
        ui.add_space(20.0);
        ui.horizontal(|ui| {
            ui.add_space(10.0);
            ui.columns(4, |columns| {
                let [col1, col2, col3, col4] = columns.get_disjoint_mut([0, 1, 2, 3]).unwrap();

                let contributions = make_peak_contributions(&info.data);

                col1.horizontal(|ui| {
                    ui.add_space(10.0);
                    add_table(
                        ui,
                        "Peak Contributions",
                        ["Location", "Peak"],
                        contributions,
                    );
                    ui.add_space(10.0);
                });
            });
            ui.add_space(10.0);
        })
    });
}

fn make_peak_contributions(data: &AccumulatedData) -> Vec<(String, String)> {
    let grouped = data
        .allocations
        .iter()
        .map(|alloc| {
            let trace = &data.traces[(alloc.trace_idx - 1) as usize];
            let ip = &data.instruction_pointers[(trace.ip_idx - 1) as usize];
            let fn_idx = match ip.frame {
                Frame::Single { function_idx } => function_idx,
                Frame::Multiple { function_idx, .. } => function_idx,
            };
            let fn_name = &data.strings[fn_idx];
            (fn_name, alloc.data.peak)
        })
        .into_grouping_map()
        .sum();

    let contributions = grouped
        .iter()
        .sorted_by(|a, b| b.1.cmp(&a.1))
        .map(|i| (i.0.to_string(), ByteSize::b(*i.1).to_string()))
        .collect::<Vec<_>>();

    contributions
}

fn add_table(
    ui: &mut Ui,
    label: &str,
    headers: [&str; 2],
    data: impl IntoIterator<Item = (String, String)>,
) {
    ui.push_id(Instant::now(), |ui| {
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
                                ui.label(a.0);
                            });
                            row.col(|ui| {
                                ui.label(a.1);
                            });
                        })
                    }
                });
        });
    });
}
