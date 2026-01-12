use crate::config::{Config, emit_div, emit_p};

pub struct VerticalSelectorItem {
    // css_sel: String,
    apply_css_rules: Vec<String>,
}

fn emit_vertical_select(
    config: &Config,
    html: &mut String,
    css: &mut String,
    opts: &[&str],
    // lookup: impl Fn(usize, &str) -> VerticalSelectorItem,
) {
    emit_div(html, "vertical-select", |html| {
        emit_p(html, "vertical-select-header border-style-dark-1", opts[0]);
        emit_div(
            html,
            "npf-select border-style-dark-1 vertical-select-group",
            |html| {
                for &opt in opts {
                    emit_p(html, "vertical-select-item", opt);
                }
            },
        )
    });
}
