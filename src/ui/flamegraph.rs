use crate::ui::widgets::flamegraph::{Flamegraph, Options};
use crate::ui::MemInfo;
use common::parser::{AccumulatedData, Allocation, Frame, InstructionPointer};
use egui::{ComboBox, Ui};
use std::iter;

struct Line {
    frames: Vec<String>,
    value: f64,
}

#[derive(PartialEq, Debug, Clone, Copy)]
enum MemoryKind {
    Peak,
    Allocations,
    Temporary,
    Leaked,
}

impl Line {
    pub fn new(value: f64) -> Self {
        Self {
            frames: Vec::new(),
            value,
        }
    }

    fn into_string(self) -> String {
        let frames = self
            .frames
            .into_iter()
            .rev()
            .collect::<Vec<String>>()
            .join(";");

        format!("{} {}", frames, self.value)
    }
}

pub struct FlamegraphPage {
    memory_kind: MemoryKind,
    peak_frame_lines: Vec<String>,
    tmp_frame_lines: Vec<String>,
    allocations_frame_lines: Vec<String>,
    leaked_frame_lines: Vec<String>,
    flamegraph: Flamegraph,
}

impl FlamegraphPage {
    pub fn new(info: &MemInfo) -> Self {
        let options = Options {
            frame_height: 20.0,
            show_info_bar: true,
        };

        let fg = Flamegraph::new(options);

        let peak_frame_lines = Self::make_frame_lines(info, |a| a.data.peak as f64);
        let tmp_frame_lines = Self::make_frame_lines(info, |a| a.data.temporary as f64);
        let allocations_frame_lines = Self::make_frame_lines(info, |a| a.data.allocations as f64);
        let leaked_frame_lines = Self::make_frame_lines(info, |a| a.data.leaked as f64);

        Self {
            memory_kind: MemoryKind::Peak,
            peak_frame_lines,
            tmp_frame_lines,
            allocations_frame_lines,
            leaked_frame_lines,
            flamegraph: fg,
        }
    }

    pub fn show(&mut self, ui: &mut Ui) {
        let prev_memory_kind = self.memory_kind;

        ComboBox::from_label("")
            .selected_text(format!("{:?}", self.memory_kind))
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut self.memory_kind, MemoryKind::Peak, "Peak");
                ui.selectable_value(&mut self.memory_kind, MemoryKind::Temporary, "Temporary");
                ui.selectable_value(&mut self.memory_kind, MemoryKind::Leaked, "Leaked");
                ui.selectable_value(
                    &mut self.memory_kind,
                    MemoryKind::Allocations,
                    "Allocations",
                );
            });

        if prev_memory_kind != self.memory_kind {
            self.flamegraph.reset();
        }

        ui.add_space(20.0);

        match self.memory_kind {
            MemoryKind::Peak => {
                let frames = self.peak_frame_lines.iter().map(|v| v.as_str());
                self.flamegraph.show(ui, frames, "bytes");
            }
            MemoryKind::Leaked => {
                let frames = self.leaked_frame_lines.iter().map(|v| v.as_str());
                self.flamegraph.show(ui, frames, "bytes");
            }
            MemoryKind::Temporary => {
                let frames = self.tmp_frame_lines.iter().map(|v| v.as_str());
                self.flamegraph.show(ui, frames, "");
            }
            MemoryKind::Allocations => {
                let frames = self.allocations_frame_lines.iter().map(|v| v.as_str());
                self.flamegraph.show(ui, frames, "");
            }
        }
    }

    fn make_frame_lines(info: &MemInfo, f: impl Fn(&Allocation) -> f64) -> Vec<String> {
        let mut lines = Vec::new();

        for alloc_info in &info.data.allocation_infos {
            let allocation = &info.data.allocations[alloc_info.allocation_idx as usize];
            let mut trace_idx = allocation.trace_idx;

            let value = f(allocation);
            let mut line = Line::new(value);

            while trace_idx != 0 {
                let trace = &info.data.traces[trace_idx as usize - 1];
                let ip_info = &info.data.instruction_pointers[trace.ip_idx as usize - 1];

                let frames = get_frames_from_ip_info(&info.data, ip_info);
                line.frames.extend(frames);

                trace_idx = trace.parent_idx;
            }
            lines.push(line.into_string());
        }

        lines
    }
}

fn get_frames_from_ip_info(data: &AccumulatedData, ip_info: &InstructionPointer) -> Vec<String> {
    iter::once(&ip_info.frame)
        .chain(ip_info.inlined.iter())
        .map(|frame| {
            let function_idx = match frame {
                Frame::Single { function_idx } => function_idx,
                Frame::Multiple { function_idx, .. } => function_idx,
            };
            data.strings[function_idx - 1].clone()
        })
        .collect()
}
