use eframe::egui::*;
use std::collections::BTreeMap;

const FRAME_V_SPACING: f32 = 4.0;
const FRAME_H_SPACING: f32 = 4.0;
const TEXT_HEIGHT: f32 = 15.0;

#[derive(Clone)]
pub struct Options {
    pub frame_height: f32,
}

#[derive(Clone, Default)]
pub struct StackFrame {
    pub label: String,
    pub value: f64,
    pub children: BTreeMap<String, StackFrame>,
}

struct Canvas {
    ctx: Context,
    rect: Rect,
    painter: Painter,
    options: Options,
}

pub fn draw_flamegraph<'a>(
    ui: &mut Ui,
    options: Options,
    frames: impl IntoIterator<Item = &'a str>,
) {
    let root_frame = build_stackframes(frames);

    ui.horizontal_centered(|ui| {
        Frame::canvas(ui.style()).show(ui, |ui| {
            let rect = ui.available_rect_before_wrap();

            let canvas = Canvas {
                ctx: ui.ctx().clone(),
                rect,
                painter: ui.painter_at(rect),
                options,
            };

            draw_root_frames(&canvas, root_frame);
        });
    });
}

fn draw_root_frames(canvas: &Canvas, root: StackFrame) {
    let min_x = canvas.rect.min.x;
    let max_x = canvas.rect.max.x;

    draw_one_frame(canvas, &root, 0, min_x, max_x);
}

fn draw_one_frame(canvas: &Canvas, frame: &StackFrame, depht: u32, min_x: f32, max_x: f32) {
    let min_y = canvas.rect.min.y + depht as f32 * (canvas.options.frame_height + FRAME_V_SPACING);

    let max_y = min_y + canvas.options.frame_height;

    let rect = Rect::from_min_max(pos2(min_x, min_y), pos2(max_x, max_y));

    canvas.painter.rect_filled(rect, 0.0, Color32::GREEN);
    let painter = canvas.painter.with_clip_rect(rect.intersect(canvas.rect));
    let text = format!("{}: {}", frame.label, frame.value);

    let text_pos = pos2(
        min_x + 4.0,
        min_y + 0.5 * (canvas.options.frame_height - TEXT_HEIGHT),
    );

    painter.text(
        text_pos,
        Align2::LEFT_TOP,
        text,
        FontId::default(),
        Color32::BLACK,
    );

    let mut child_min_x = min_x;
    let length = max_x - min_x;
    for (_, child) in &frame.children {
        let child_value = frame.value.min(child.value);

        let child_max_x = max_x.min(child_min_x + (child_value / frame.value) as f32 * length);

        draw_one_frame(canvas, child, depht + 1, child_min_x, child_max_x);

        child_min_x = child_max_x + FRAME_H_SPACING;
    }
}

fn build_stackframes<'a>(chains: impl IntoIterator<Item = &'a str>) -> StackFrame {
    let mut root = StackFrame::default();
    root.label = "all".to_string();

    for chain in chains {
        let (frames, value) = chain.rsplit_once(" ").unwrap();
        let value = value.parse::<f64>().unwrap();

        root.value += value;
        fill_children(&mut root, frames, value);
    }

    root
}

fn fill_children(sf: &mut StackFrame, frames: &str, value: f64) {
    let Some((frame, frames)) = frames.split_once(";") else {
        sf.label = frames.to_string();
        sf.value += value;
        return;
    };

    let next = sf.children.entry(frame.to_string()).or_insert(StackFrame {
        label: frame.to_string(),
        value: 0.0,
        children: Default::default(),
    });

    next.value += value;

    fill_children(next, frames, value);
}
