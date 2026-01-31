use crate::config::{Config, emit_div, emit_img};

pub struct ToolbarBuilder {
    groups: Vec<ToolbarGroup>,
}

impl ToolbarBuilder {
    pub fn new() -> Self {
        Self { groups: Vec::new() }
    }
    pub fn group(&mut self, func: impl Fn(&mut ToolbarGroup) -> &mut ToolbarGroup) -> &mut Self {
        self.groups.push(ToolbarGroup::new());
        func(self.groups.last_mut().unwrap());
        self
    }

    pub fn build(&self, html: &mut String, _css: &mut String, _config: &Config) {
        let emit_item = |html: &mut String, item: &ToolbarItem| {
            match item {
                ToolbarItem::Normal(item) => {
                    emit_div(html, "toolbar-item-outer", |html| {
                        emit_div(html, &format!("toolbar-item {}", item.class), |html| {
                            html.push_str(&format!(
                                r##"
                                        <img src="{}" />
                                        <p>{}</p>
                                    "##,
                                item.icon, item.name
                            ));
                        });
                        emit_div(html, "toolbar-item toolbar-single-click", |_| {});
                    });
                }
                ToolbarItem::Html(item) => {
                    html.push_str(&item.html);
                }
            };
        };
        emit_div(html, "toolbar-anchor", |html| {
            emit_div(html, "toolbar border-style-light-1", |html| {
                for (i, groupp) in self.groups.iter().enumerate() {
                    if i != 0 {
                        emit_div(html, "toolbar-sep", |_| {});
                    }
                    for item in groupp.items.iter() {
                        emit_item(html, item);
                    }
                }
                emit_div(html, "toolbar-overflow", |html| {
                    emit_div(html, "tbo-arrow", |html| {
                        emit_img(html, "", "@icon:arrow2");
                    });
                    emit_div(html, "tbo-arrow", |html| {
                        emit_img(html, "", "@icon:arrow2");
                    });
                });
            });
            emit_div(html, "tbo-menu border-style-dark-3", |html| {
                emit_div(html, "tbo-enabled-arrow", |html| {
                    emit_img(html, "", "@icon:arrow2");
                });
                for group in self.groups.iter() {
                    for item in group.items.iter() {
                        emit_item(html, item);
                    }
                }
            });
        });
    }
}

pub struct ToolbarGroup {
    items: Vec<ToolbarItem>,
}

impl ToolbarGroup {
    fn new() -> Self {
        Self { items: Vec::new() }
    }
    pub fn item(&mut self, name: &str, icon: &str, class: &str) -> &mut Self {
        self.items.push(ToolbarItem::new(name, icon, class));
        self
    }
    pub fn item_html(&mut self, mut func: impl FnMut(&mut String)) -> &mut Self {
        let mut html = String::new();
        func(&mut html);
        self.items.push(ToolbarItem::new_html(html));
        self
    }
}

pub enum ToolbarItem {
    Normal(ToolbarItemNormal),
    Html(ToolbarItemHtml),
}

pub struct ToolbarItemNormal {
    name: String,
    class: String,
    icon: String,
}
pub struct ToolbarItemHtml {
    html: String,
}

impl ToolbarItem {
    fn new(name: &str, icon: &str, class: &str) -> Self {
        Self::Normal(ToolbarItemNormal {
            name: name.into(),
            icon: icon.into(),
            class: class.into(),
        })
    }
    fn new_html(html: String) -> Self {
        Self::Html(ToolbarItemHtml { html: html })
    }
}
