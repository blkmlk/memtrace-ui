use crate::ui::widgets::flamegraph::{draw_flamegraph, Options};
use crate::ui::MemInfo;
use common::parser::{AccumulatedData, Frame, InstructionPointer};
use egui::Ui;
use std::iter;

struct Line {
    frames: Vec<String>,
    value: f64,
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

pub fn show_ui(ui: &mut Ui, info: &MemInfo) {
    let options = Options { frame_height: 20.0 };
    let mut lines = Vec::new();

    for alloc_info in &info.data.allocation_infos {
        let allocation = &info.data.allocations[alloc_info.allocation_idx as usize];
        let mut trace_idx = allocation.trace_idx;

        let mut line = Line::new(alloc_info.size as f64);
        while trace_idx != 0 {
            let trace = &info.data.traces[trace_idx as usize - 1];
            let ip_info = &info.data.instruction_pointers[trace.ip_idx as usize - 1];

            let frames = get_frames_from_ip_info(&info.data, ip_info);
            line.frames.extend(frames);

            trace_idx = trace.parent_idx;
        }
        lines.push(line.into_string());
    }

    draw_flamegraph(ui, options, lines.iter().map(|v| v.as_str()));
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
