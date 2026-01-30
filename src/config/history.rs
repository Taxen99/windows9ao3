use std::sync::OnceLock;

use crate::config::{Config, emit_div};

pub struct History {
    items: Vec<HistoryItem>,
    default_id: u64,
}

static MAKE_SURE_WE_ONLY_USE_HISTORY_ONCE_LOL: OnceLock<()> = OnceLock::new();

impl History {
    pub fn new(items: Vec<HistoryItem>, default_id: u64) -> Self {
        MAKE_SURE_WE_ONLY_USE_HISTORY_ONCE_LOL
            .set(())
            .expect("can't use history more than once at the moment.");
        Self { items, default_id }
    }
    pub fn emit_stack(&self, html: &mut String, css: &mut String, _config: &Config) {
        emit_div(html, "history", |html| {
            for item in self.items.iter() {
                css.push_str(&format!(
                    r##"
                    .main:has(.history-trigger-{0}:hover:active) .history-item-{0} {{
                        transition: 0s;
                        left: 0.01px;
                        top: 0.01px;
                        z-index: 2147483641;
                    }}
                    "##,
                    item.id
                ));
            }
            emit_div(html, "history-items", |html| {
                emit_div(html, "history-none-back", |_| ());
                emit_div(html, "history-none-forward", |_| ());
                for item in &self.items {
                    emit_div(
                        html,
                        &format!("history-item history-item-{}", item.id),
                        |html| {
                            emit_div(html, "history-item-reg", |_| ());
                            emit_div(html, "history-item-move", |_| ());
                            emit_div(html, "history-item-check-back", |_| ());
                            emit_div(html, "history-item-check-forward", |_| ());
                        },
                    );
                    for rule in &item.rules {
                        css.push_str(&format!(
                            r##".main:has(.history-item-{0} .history-item-reg:hover) {1}"##,
                            item.id, rule
                        ));
                    }
                }
            });
        });
        let default_item = self.items.iter().find(|x| x.id == self.default_id).unwrap();
        for rule in &default_item.rules {
            // three .main:s to increase specificity!
            css.push_str(&format!(r##".main.main.main:has(.onload:hover) {}"##, rule));
            css.push_str(&format!(
                r##".main:has(.history-none-back:hover) {}"##,
                rule
            ));
        }
    }
}

pub struct HistoryItem {
    pub id: u64,
    pub rules: Vec<String>,
}
