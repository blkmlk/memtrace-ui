use crate::ui::MemInfo;
use bytesize::ByteSize;
use egui::*;
use egui_extras::{Column, TableBuilder};
use egui_ltreeview::{Action, NodeBuilder, TreeView, TreeViewBuilder};
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::ops::Deref;
use std::rc::Rc;
use std::{fs, iter};

const MIN_PANEL_WIDTH: f32 = 500.0;

#[derive(Debug, Clone)]
struct StackNode {
    info: Rc<RefCell<StackInfo>>,
    children: BTreeMap<String, StackNode>,
}

#[derive(Debug, Clone, Default)]
struct StackInfo {
    id: u32,
    name: String,
    file_name: String,
    line_number: u32,
    peaked: u64,
    leaked: u64,
    allocations: u64,
    temporary: u64,
}

pub struct TopDown {
    panel_width: f32,
    root_node: StackNode,
    stack_info_by_id: HashMap<u32, Rc<RefCell<StackInfo>>>,
    selected_stack_info_id: u32,
    code_loader: CodeLoader,
}

impl TopDown {
    pub fn new(info: &MemInfo) -> Self {
        let (root_stack_dir, file_info_by_id) = make_stack_dirs(info);

        Self {
            panel_width: MIN_PANEL_WIDTH,
            root_node: root_stack_dir,
            stack_info_by_id: file_info_by_id,
            selected_stack_info_id: 0,
            code_loader: CodeLoader::new(),
        }
    }

    pub fn show(&mut self, ui: &mut Ui) {
        let available_height = ui.available_height();
        let max_width = ui.available_width() / 2.0;
        let max_height = ui.available_height();

        let style = ui.style();
        let font_size = style.text_styles.get(&TextStyle::Body).unwrap().size;

        ui.horizontal(|ui| {
            ui.horizontal(|ui| {
                let id = ui.make_persistent_id("left_panel");

                let (_, actions) = TreeView::new(id)
                    .max_width(self.panel_width)
                    .max_height(available_height)
                    .allow_multi_selection(false)
                    .show(ui, |view| {
                        self.show_node(view, &self.root_node);
                    });

                for action in actions {
                    match action {
                        Action::SetSelected(ids) => {
                            assert_eq!(ids.len(), 1);
                            let info = self.stack_info_by_id.get(&ids[0]).unwrap();
                            self.selected_stack_info_id = info.borrow().id;
                        }
                        Action::Move(_) => {}
                        Action::Drag(_) => {}
                        Action::Activate(_) => {}
                    }
                }
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
                let info = self
                    .stack_info_by_id
                    .get(&self.selected_stack_info_id)
                    .unwrap();

                let offset = (max_height / font_size) as u32;
                self.code_loader.show(ui, info.borrow(), offset);
            });
        });
    }

    fn show_node(&self, view: &mut TreeViewBuilder<u32>, node: &StackNode) {
        if node.children.is_empty() {
            view.leaf(node.info.borrow().id, &node.info.borrow().name);
        } else {
            view.node(
                NodeBuilder::dir(node.info.borrow().id)
                    .label(&node.info.borrow().name)
                    .default_open(false)
                    .activatable(true),
            );
            for child in node.children.values() {
                self.show_node(view, child);
            }
            view.close_dir()
        }
    }
}

fn make_stack_dirs(info: &MemInfo) -> (StackNode, HashMap<u32, Rc<RefCell<StackInfo>>>) {
    let mut global_id = 0;
    let mut mapped = HashMap::new();

    let root_info = Rc::new(RefCell::new(StackInfo {
        name: "all".to_string(),
        ..Default::default()
    }));

    let mut root = StackNode {
        info: root_info.clone(),
        children: BTreeMap::new(),
    };
    mapped.insert(global_id, root_info);

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

                    let info = Rc::new(RefCell::new(StackInfo {
                        id: global_id,
                        name: info.data.strings[fn_idx - 1].clone(),
                        file_name,
                        line_number: parent_ln,
                        ..Default::default()
                    }));

                    let node = StackNode {
                        info: info.clone(),
                        children: BTreeMap::new(),
                    };

                    mapped.insert(node.info.borrow().id, info);

                    node
                });

                {
                    let mut info = child.info.borrow_mut();
                    info.peaked += allocation.data.peak;
                    info.leaked += allocation.data.leaked;
                    info.allocations += allocation.data.allocations;
                    info.temporary += allocation.data.temporary;
                }

                parent_file_idx = *file_idx;
                parent_ln = *ln;

                current = child;
            }
        }
    }

    (root, mapped)
}

struct CodeLoader {
    mapped: HashMap<String, String>,
}

impl CodeLoader {
    pub fn new() -> Self {
        Self {
            mapped: HashMap::new(),
        }
    }

    pub fn show(&mut self, ui: &mut Ui, stack_info: impl Deref<Target = StackInfo>, offset: u32) {
        if !self.mapped.contains_key(&stack_info.file_name) {
            let Ok(code) = fs::read_to_string(&stack_info.file_name) else {
                return;
            };
            self.mapped.insert(stack_info.file_name.to_string(), code);
        };

        let code = self.mapped.get(&stack_info.file_name).unwrap();

        let min_offset = stack_info.line_number.saturating_sub(offset / 2) as usize;
        let max_offset = (stack_info.line_number + offset / 2) as usize;

        let lines = code
            .lines()
            .enumerate()
            .filter(|(i, _)| *i + 1 >= min_offset && *i + 1 <= max_offset);

        TableBuilder::new(ui)
            .resizable(false)
            .cell_layout(Layout::left_to_right(Align::Center))
            .column(Column::initial(20.0))
            .column(Column::initial(100.0).at_most(100.0))
            .column(Column::remainder().at_least(50.0))
            .body(|mut body| {
                for (i, line) in lines {
                    let number = (i + 1) as u32;
                    body.row(15.0, |mut row| {
                        row.col(|ui| {
                            ui.label(format!("{}", number));
                        });
                        row.col(|ui| {
                            ui.code(line);
                        });
                        if number == stack_info.line_number {
                            row.col(|ui| {
                                ui.label(ByteSize::b(stack_info.peaked).to_string());
                            });
                        }
                    });
                }
            });
    }
}
