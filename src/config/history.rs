use crate::config::{Config, emit_div};

pub struct History {
    items: Vec<HistoryItem>,
}

impl History {
    pub fn new(items: Vec<HistoryItem>) -> Self {
        Self { items }
    }
    pub fn emit_stack(&self, html: &mut String, css: &mut String, config: &Config) {
        emit_div(html, "history", |html| {
            emit_div(html, "history-l-shifter", |_| ());
            emit_div(html, "history-r-shifter", |_| ());
            html.push_str(&format!(
                r##"
                <button class="history-back">&lt;-</button>
                <button class="history-forward">-&gt;</button>
            "##
            ));
            for (i, item) in self.items.iter().enumerate() {
                html.push_str(&format!(
                    r##"
                    <button class="history-trigger-{1}">{0}:{1}</button>
                "##,
                    i, item.id
                ));
                css.push_str(&format!(
                    r##"
                    .history-trigger-{0}:hover:active {{
                        background-color: blue;
                    }}
                    .history:has(.history-trigger-{0}:hover:active) .history-item-{0} {{
                        transition: 0s;
                        left: 100vw;
                    }}
                    .history:has(.history-trigger-{0}:hover:active) .history-l-shifter {{
                        transition: 0s;
                        left: 0;
                    }}
                    "##,
                    item.id
                ));
            }
            for item in &self.items {
                emit_div(
                    html,
                    &format!("history-item history-item-{}", item.id),
                    |_| (),
                );
            }
        });
    }
}

pub struct HistoryItem {
    pub id: u64,
    pub rule: String,
}
