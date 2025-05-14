use egui::RichText;
use egui::Ui;

pub fn add_key_value(ui: &mut Ui, key: &str, value: impl ToString) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(format!("{key}:")).strong());
        ui.label(value.to_string());
    });
}
