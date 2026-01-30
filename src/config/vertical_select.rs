use crate::config::{Config, emit_div, emit_p};

pub fn emit_vertical_select(
    _config: &Config,
    html: &mut String,
    _css: &mut String,
    opts: &[&str],
    default: &str,
) {
    assert!(opts.contains(&default));
    emit_div(html, "vertical-select", |html| {
        emit_p(
            html,
            "vertical-select-header vertical-select-header-fake border-style-dark-1",
            "fake news",
        );
        emit_div(html, "border-style-dark-1 vertical-select-group", |html| {
            for &opt in opts {
                if opt == default {
                    emit_div(
                        html,
                        "vertical-select-item vertical-select-item-default",
                        |html| emit_p(html, "", opt),
                    );
                    emit_p(
                        html,
                        "vertical-select-header vertical-select-header-default border-style-dark-1",
                        opt,
                    );
                } else {
                    emit_div(html, "vertical-select-item", |html| emit_p(html, "", opt));
                    emit_p(html, "vertical-select-header border-style-dark-1", opt);
                }
            }
        })
    });
}
