use crate::ui::helpers::key_value;
use crate::ui::AnalyzedInfo;
use bytesize::ByteSize;
use egui::{Layout, Ui};

pub fn show(ui: &mut Ui, info: &AnalyzedInfo) {
    ui.with_layout(Layout::default(), |ui| {
        ui.add_space(20.0);
        ui.horizontal(|ui| {
            ui.add_space(20.0);
            ui.columns(3, |columns| {
                let [c1, c2, c3] = columns.get_disjoint_mut([0, 1, 2]).unwrap();

                let total_ram = info.data.page_size * info.data.pages;

                key_value(c1, "application", &info.app_name);
                key_value(c1, "total runtime", format!("{:?}", info.data.duration));
                key_value(c1, "total system memory", ByteSize::b(total_ram));

                key_value(
                    c2,
                    "calls to allocation functions",
                    info.data.total.allocations,
                );
                key_value(c2, "temporary allocations", info.data.total.temporary);

                key_value(
                    c3,
                    "peak heap memory consumption",
                    ByteSize::b(info.data.total.peak),
                );
                key_value(c3, "peak RSS", ByteSize::b(info.data.peak_rss));
                key_value(c3, "total memory leaked", info.data.total.leaked);
            });
            ui.add_space(20.0);
        });
        ui.add_space(20.0);
        ui.separator()
    });
}
