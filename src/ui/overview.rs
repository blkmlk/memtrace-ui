use crate::ui::helpers::add_key_value;
use crate::ui::MemInfo;
use bytesize::ByteSize;
use eframe::emath::Align;
use egui::{Layout, Ui};
use egui_extras::{Column, TableBuilder};
use itertools::Itertools;
use memtrack_utils::parser::{AccumulatedData, Frame};
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

                let peaks = make_top_peaks(&info.data);
                col1.horizontal(|ui| {
                    ui.add_space(10.0);
                    add_table(ui, "Peak Contributions", ["Location", "Peak"], peaks);
                    ui.add_space(10.0);
                });

                let leaks = make_top_leaks(&info.data);
                col2.horizontal(|ui| {
                    ui.add_space(10.0);
                    add_table(ui, "Largest Memory Leaks", ["Location", "Leaked"], leaks);
                    ui.add_space(10.0);
                });

                let allocations = make_top_allocations(&info.data);
                col3.horizontal(|ui| {
                    ui.add_space(10.0);
                    add_table(
                        ui,
                        "Most Memory Allocations",
                        ["Location", "Allocations"],
                        allocations,
                    );
                    ui.add_space(10.0);
                });

                let tmp_allocations = make_top_tmp_allocations(&info.data);
                col4.horizontal(|ui| {
                    ui.add_space(10.0);
                    add_table(
                        ui,
                        "Most Temporary Allocations",
                        ["Location", "Temporary"],
                        tmp_allocations,
                    );
                    ui.add_space(10.0);
                });
            });
            ui.add_space(10.0);
        })
    });
}

fn make_top_peaks(data: &AccumulatedData) -> Vec<(String, String)> {
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

fn make_top_leaks(data: &AccumulatedData) -> Vec<(String, String)> {
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
            (fn_name, alloc.data.leaked)
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

fn make_top_allocations(data: &AccumulatedData) -> Vec<(String, String)> {
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
            (fn_name, alloc.data.allocations)
        })
        .into_grouping_map()
        .sum();

    let contributions = grouped
        .iter()
        .sorted_by(|a, b| b.1.cmp(&a.1))
        .map(|i| (i.0.to_string(), i.1.to_string()))
        .collect::<Vec<_>>();

    contributions
}

fn make_top_tmp_allocations(data: &AccumulatedData) -> Vec<(String, String)> {
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
            (fn_name, alloc.data.temporary)
        })
        .into_grouping_map()
        .sum();

    let contributions = grouped
        .iter()
        .sorted_by(|a, b| b.1.cmp(&a.1))
        .map(|i| (i.0.to_string(), i.1.to_string()))
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
