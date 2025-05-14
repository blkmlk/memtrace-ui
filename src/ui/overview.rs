use crate::ui::helpers::add_key_value;
use crate::ui::MemInfo;
use bytesize::ByteSize;
use egui::{Layout, Ui};

pub fn show(ui: &mut Ui, info: &MemInfo) {
    ui.with_layout(Layout::default(), |ui| {
        ui.add_space(20.0);
        ui.horizontal(|ui| {
            ui.add_space(20.0);
            ui.columns(3, |columns| {
                let [col1, col2, col3] = columns.get_disjoint_mut([0, 1, 2]).unwrap();

                let total_ram = info.data.page_size * info.data.pages;

                add_key_value(col1, "application", &info.app_name);
                add_key_value(col1, "total runtime", format!("{:?}", info.data.duration));
                add_key_value(col1, "total system memory", ByteSize::b(total_ram));

                add_key_value(
                    col2,
                    "calls to allocation functions",
                    info.data.total.allocations,
                );
                add_key_value(col2, "temporary allocations", info.data.total.temporary);

                add_key_value(
                    col3,
                    "peak heap memory consumption",
                    ByteSize::b(info.data.total.peak),
                );
                add_key_value(col3, "peak RSS", ByteSize::b(info.data.peak_rss));
                add_key_value(col3, "total memory leaked", info.data.total.leaked);
            });
            ui.add_space(20.0);
        });
        ui.add_space(20.0);
        ui.separator()
    });
}
