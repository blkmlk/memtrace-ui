use crate::ui::MemInfo;
use egui::*;
use egui_ltreeview::{NodeBuilder, TreeView, TreeViewBuilder, TreeViewState};
use std::collections::BTreeMap;
use std::iter;

const MIN_PANEL_WIDTH: f32 = 200.0;

pub struct TopDown {
    panel_width: f32,
    root_stack_dir: StackDir,
}

#[derive(Debug)]
struct StackDir {
    id: u32,
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
                let mut state = TreeViewState::default();

                TreeView::new(id)
                    .max_width(self.panel_width)
                    .max_height(available_height)
                    .allow_multi_selection(false)
                    .show_state(ui, &mut state, |view| {
                        self.show_dir(view, &self.root_stack_dir);
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

    fn show_dir(&self, view: &mut TreeViewBuilder<u32>, dir: &StackDir) {
        if dir.children.is_empty() {
            view.leaf(dir.id, &dir.name);
        } else {
            view.node(
                NodeBuilder::dir(dir.id)
                    .label(&dir.name)
                    .default_open(false)
                    .activatable(true),
            );
            for child in dir.children.values() {
                self.show_dir(view, child);
            }
            view.close_dir()
        }
    }
}

fn make_stack_dirs(info: &MemInfo) -> StackDir {
    let mut global_id = 0;

    let mut root = StackDir {
        id: global_id,
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

        let mut parent_file_idx = 0;
        let mut parent_ln = 0;
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

                let key = format!("{}:{}:{}", fn_idx, parent_file_idx, parent_ln);

                let child = current.children.entry(key).or_insert_with(|| {
                    let file_name = if parent_file_idx == 0 {
                        String::new()
                    } else {
                        info.data.strings[parent_file_idx - 1].clone()
                    };

                    global_id += 1;

                    StackDir {
                        id: global_id,
                        name: info.data.strings[fn_idx - 1].clone(),
                        file_name,
                        line_number: parent_ln,
                        children: BTreeMap::new(),
                    }
                });

                parent_file_idx = *file_idx;
                parent_ln = *ln;

                current = child;
            }
        }
    }

    root
}
