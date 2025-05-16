use crate::ui::widgets::flamegraph::{draw_flamegraph, Options, StackFrame};
use crate::ui::MemInfo;
use egui::Ui;

pub fn show_ui(ui: &mut Ui, info: &MemInfo) {
    let options = Options { frame_height: 20.0 };

    let root = [
        StackFrame {
            label: "main".to_string(),
            value: 5.0,
            children: vec![
                StackFrame {
                    label: "ch1".to_string(),
                    value: 3.0,
                    children: vec![],
                },
                StackFrame {
                    label: "ch2".to_string(),
                    value: 2.0,
                    children: vec![],
                },
            ],
        },
        StackFrame {
            label: "side".to_string(),
            value: 7.0,
            children: vec![],
        },
    ];

    draw_flamegraph(ui, options, &root);
}
