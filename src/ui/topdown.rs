use crate::ui::MemInfo;
use common::parser::{AccumulatedData, InstructionPointer};
use egui::*;
use egui_ltreeview::TreeView;
use std::collections::BTreeMap;
use std::iter;

const MIN_PANEL_WIDTH: f32 = 200.0;

pub struct TopDown {
    panel_width: f32,
    root_stack_dir: StackDir,
}

struct StackDir {
    name: String,
    file_name: String,
    line_number: u32,
    children: BTreeMap<String, StackDir>,
}

impl TopDown {
    pub fn new(info: &MemInfo) -> Self {
        let root_stack_dir = make_stack_dirs(info);
        Self {
            panel_width: MIN_PANEL_WIDTH,
            root_stack_dir,
        }
    }

    pub fn show(&mut self, ui: &mut Ui) {
        let available_height = ui.available_height();
        let max_width = ui.available_width() / 2.0;

        ui.horizontal(|ui| {
            ui.horizontal(|ui| {
                let id = ui.make_persistent_id("left_panel");
                TreeView::new(id)
                    .max_width(self.panel_width)
                    .max_height(available_height)
                    .allow_multi_selection(false)
                    .show(ui, |view| {
                        view.dir(0, "l1");
                        view.leaf(1, "l2");
                        view.leaf(2, "l3");
                        view.leaf(3, "l4");
                    });
            });

            let separator_response = ui
                .allocate_exact_size(vec2(4.0, available_height), Sense::drag())
                .1
                .on_hover_cursor(CursorIcon::ResizeHorizontal);

            let stroke = Stroke::new(
                3.0,
                ui.style().visuals.widgets.noninteractive.bg_stroke.color,
            );
            let center_x = separator_response.rect.center().x;

            ui.painter().line_segment(
                [
                    pos2(center_x, separator_response.rect.top()),
                    pos2(center_x, separator_response.rect.bottom()),
                ],
                stroke,
            );

            let ctx = ui.ctx();
            if separator_response.dragged() {
                self.panel_width += ctx.input(|i| i.pointer.delta().x);
                self.panel_width = self.panel_width.clamp(MIN_PANEL_WIDTH, max_width);
            }

            ui.vertical(|ui| {
                ui.label("Main content area");
            });
        });
    }
}

fn make_stack_dirs(info: &MemInfo) -> StackDir {
    let mut root = StackDir {
        name: "all".to_string(),
        file_name: "".to_string(),
        line_number: 0,
        children: BTreeMap::new(),
    };

    for alloc_info in &info.data.allocation_infos {
        let allocation = &info.data.allocations[alloc_info.allocation_idx as usize];
        let mut trace_idx = allocation.trace_idx;

        let mut current = &mut root;
        while trace_idx != 0 {
            let trace = &info.data.traces[trace_idx as usize - 1];
            let ip_info = &info.data.instruction_pointers[trace.ip_idx as usize - 1];

            let frames = get_frames_from_ip_info(&info.data, ip_info);

            trace_idx = trace.parent_idx;
        }
    }
    todo!()
}

fn get_frames_from_ip_info(data: &AccumulatedData, ip_info: &InstructionPointer) -> Vec<String> {
    iter::once(&ip_info.frame)
        .chain(&ip_info.inlined)
        .map(|frame| {
            let function_idx = match frame {
                common::parser::Frame::Single { function_idx } => function_idx,
                common::parser::Frame::Multiple { function_idx, .. } => function_idx,
            };
            data.strings[function_idx - 1].clone()
        })
        .collect()
}
