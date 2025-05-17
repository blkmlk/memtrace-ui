use eframe::egui::*;
use egui::ecolor::Hsva;
use std::collections::BTreeMap;
use std::hash::{DefaultHasher, Hash, Hasher};

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
    response: Response,
    rect: Rect,
    painter: Painter,
}

pub struct Flamegraph {
    options: Options,
}

impl Flamegraph {
    pub fn new<'a>(opts: Options) -> Self {
        Self { options: opts }
    }

    pub fn show<'a>(&mut self, ui: &mut Ui, frames: impl IntoIterator<Item = &'a str>) {
        ui.horizontal_centered(|ui| {
            let root = build_stackframes(frames);

            Frame::canvas(ui.style()).show(ui, |ui| {
                let rect = ui.available_rect_before_wrap();
                let response = ui.interact(rect, ui.id().with("canvas"), Sense::click_and_drag());

                let canvas = Canvas {
                    ctx: ui.ctx().clone(),
                    response,
                    rect,
                    painter: ui.painter_at(rect),
                };

                self.draw(&canvas, &root);
            });
        });
    }

    fn draw(&mut self, canvas: &Canvas, root: &StackFrame) {
        let min_x = canvas.rect.min.x;
        let max_x = canvas.rect.max.x;

        self.draw_one_frame(canvas, root, 0, min_x, max_x);
    }

    fn draw_one_frame(
        &mut self,
        canvas: &Canvas,
        frame: &StackFrame,
        depth: u32,
        min_x: f32,
        max_x: f32,
    ) {
        let min_y =
            canvas.rect.min.y + depth as f32 * (self.options.frame_height + FRAME_V_SPACING);

        let max_y = min_y + self.options.frame_height;

        let rect = Rect::from_min_max(pos2(min_x, min_y), pos2(max_x, max_y));

        let is_hovered = if let Some(mouse_pos) = canvas.response.hover_pos() {
            rect.contains(mouse_pos)
        } else {
            false
        };

        let mut rect_color = make_frame_color(frame.value, depth, min_x, max_x);

        if is_hovered {
            rect_color = saturate(rect_color, 0.3);
        };

        canvas.painter.rect_filled(rect, 0.0, rect_color);
        let painter = canvas.painter.with_clip_rect(rect.intersect(canvas.rect));
        let text = format!("{}: {}", frame.label, frame.value);

        let text_pos = pos2(
            min_x + 4.0,
            min_y + 0.5 * (self.options.frame_height - TEXT_HEIGHT),
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

            self.draw_one_frame(canvas, child, depth + 1, child_min_x, child_max_x);

            child_min_x = child_max_x + FRAME_H_SPACING;
        }
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

pub fn make_frame_color(value: f64, depth: u32, min_x: f32, max_x: f32) -> Color32 {
    let mut hasher = DefaultHasher::new();
    (value.to_bits(), depth, min_x.to_bits(), max_x.to_bits()).hash(&mut hasher);
    let hash = hasher.finish();

    let hue_variation = 0.3 + ((hash & 0xFF) as f32 / 255.0) * 0.7; // [0.3, 1.0]
    let sat_variation = 0.4 + (((hash >> 8) & 0x7F) as f32 / 127.0) * 0.2; // [0.4, 0.6]
    let val_variation = ((hash >> 16) & 0x7F) as f32 / 255.0; // [0.0, 1.0]

    // Clamp hue to greenish range: 100°–160°
    let hue_deg = 100.0 + hue_variation * 60.0;
    let hue = hue_deg / 360.0;

    // Base saturation/brightness with slight noise
    let saturation = 0.6 + sat_variation * 0.4; // [0.6, 1.0]
    let brightness = 0.6 + val_variation * 0.3; // [0.6, 0.9]

    let hsva = Hsva {
        h: hue,
        s: saturation.clamp(0.0, 1.0),
        v: brightness.clamp(0.0, 1.0),
        a: 1.0,
    };

    Color32::from(hsva)
}

fn saturate(color: Color32, factor: f32) -> Color32 {
    let mut hsv = Hsva::from(color);
    hsv.s = (hsv.s * (1.0 + factor)).clamp(0.0, 1.0);
    hsv.v = (hsv.v * (1.0 - factor)).clamp(0.0, 1.0);
    Color32::from(hsv)
}
