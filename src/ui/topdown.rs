use crate::ui::MemInfo;
use egui::*;
use egui_ltreeview::TreeView;
use std::collections::BTreeMap;
use std::iter;

const MIN_PANEL_WIDTH: f32 = 200.0;

pub struct TopDown {
    panel_width: f32,
    root_stack_dir: StackDir,
}

#[derive(Debug)]
struct StackDir {
    name: String,
    file_name: String,
    line_number: u32,
    children: BTreeMap<String, StackDir>,
}

impl TopDown {
    pub fn new(info: &MemInfo) -> Self {
        let root_stack_dir = make_stack_dirs(info);

        println!("dirs: {:#?}", root_stack_dir);

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

    let mut ip_idxs = vec![];
    for alloc_info in &info.data.allocation_infos {
        let allocation = &info.data.allocations[alloc_info.allocation_idx as usize];

        let mut trace_idx = allocation.trace_idx;
        ip_idxs.clear();
        while trace_idx != 0 {
            let trace = &info.data.traces[trace_idx as usize - 1];
            ip_idxs.push(trace.ip_idx);
            trace_idx = trace.parent_idx;
        }

        let mut current = &mut root;
        for ip_idx in ip_idxs.iter().rev() {
            let ip_info = &info.data.instruction_pointers[*ip_idx as usize - 1];

            for frame in ip_info.inlined.iter().chain(iter::once(&ip_info.frame)) {
                let (fn_idx, file_idx, ln) = match frame {
                    common::parser::Frame::Single { function_idx } => (function_idx, &0, &0),
                    common::parser::Frame::Multiple {
                        function_idx,
                        file_idx,
                        line_number,
                    } => (function_idx, file_idx, line_number),
                };

                let key = format!("{}:{}:{}", info.data.strings[fn_idx - 1], file_idx, ln);
                // println!("key: {}", key);

                let child = current.children.entry(key).or_insert_with(|| {
                    let file_name = if *file_idx == 0 {
                        "".to_string()
                    } else {
                        info.data.strings[file_idx - 1].clone()
                    };

                    StackDir {
                        name: info.data.strings[fn_idx - 1].clone(),
                        file_name,
                        line_number: *ln,
                        children: BTreeMap::new(),
                    }
                });

                current = child;
            }
        }
    }

    root
}
