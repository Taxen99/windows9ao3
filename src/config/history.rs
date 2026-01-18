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
            // emit_div(html, "history-l-shifter", |_| ());
            // emit_div(html, "history-r-shifter", |_| ());
            html.push_str(&format!(
                r##"
                <button class="history-back">&lt;-</button>
                <button class="history-forward">-&gt;</button>
            "##
            ));
            for (i, item) in self.items.iter().enumerate() {
                html.push_str(&format!(
                    r##"
                    <button class="history-trigger-{1} history-trigger">{0}:{1}</button>
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
                        left: 0.01px;
                        top: 0.01px;
                        z-index: 2147483641;
                    }}
                    "##,
                    item.id
                ));
                // .history:has(.history-trigger-{0}:hover:active) .history-l-shifter {{
                //     transition: 0s;
                //     left: 0;
                // }}
            }
            emit_div(html, "cur-display", |html| {
                for (_, item) in self.items.iter().enumerate() {
                    emit_div(html, &format!("cur-{} cur", item.id), |html| {
                        html.push_str(&format!("Current: {}", item.id));
                    });
                }
            });
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
                            // emit_div(html, "history-item-check-back", |_| ());
                            emit_div(html, "history-item-check-forward", |_| ());
                        },
                    );
                    css.push_str(&format!(
                        r##"
                        .history:has(.history-item-{0} .history-item-reg:hover) .cur-{0} {{
                            transition: 0s;
                            left: 0px;
                        }}
                        .history:has(.history-item-{0} .history-item-reg:hover) .cur:not(.cur-{0}) {{
                            transition: 0s;
                            left: -100000.01px;
                        }}
                    "##,
                        item.id
                    ));
                }
            });
        });
    }
}

pub struct HistoryItem {
    pub id: u64,
    pub rule: String,
}
