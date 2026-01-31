use enum_as_inner::EnumAsInner;

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

    pub fn build(&self, html: &mut String, css: &mut String, _config: &Config) {
        let emit_item = |html: &mut String, css: &mut String, item: &ToolbarItem| {
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
                ToolbarItem::ToggleGroup(item) => {
                    assert!(!item.class.contains(' '), "this would fuck us over");
                    css.push_str(&format!(
                        r##"
                    .toolbar-anchor:has(.{0}:first-child:active) .toolbar-item-enabled.{0} {{
                        transition: 0s;
                        z-index: 2;
                    }}
                    "##,
                        item.class
                    ));
                    emit_div(
                        html,
                        &format!(
                            "toolbar-item-outer toolbar-toggle-group toolbar-toggle-group-{}",
                            item.toggle_group
                        ),
                        |html| {
                            emit_div(html, &format!("toolbar-item {}", item.class), |html| {
                                html.push_str(&format!(
                                    r##"
                                        <img src="{}" />
                                        <p>{}</p>
                                    "##,
                                    item.icon, item.name
                                ));
                            });
                            emit_div(
                                html,
                                &format!("toolbar-item toolbar-item-enabled {}", item.class),
                                |html| {
                                    html.push_str(&format!(
                                        r##"
                                        <img src="{}" />
                                        <p>{}</p>
                                    "##,
                                        item.icon, item.name
                                    ));
                                },
                            );
                            emit_div(html, "toolbar-item toolbar-single-click", |_| {});
                        },
                    );
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
                        emit_item(html, css, item);
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
                        // we don't want to emit the same css twice!
                        let mut garbage_css = String::new();
                        emit_item(html, &mut garbage_css, item);
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
        Self {
            items: Vec::new(),
            // select: false,
            // class: "".into(),
        }
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
    pub fn item_toggle_group(
        &mut self,
        name: &str,
        icon: &str,
        class: &str,
        toggle_group: &str,
    ) -> &mut Self {
        self.items.push(ToolbarItem::new_toggle_group(
            name,
            icon,
            class,
            toggle_group,
        ));
        self
    }
    // pub fn select(&mut self, select: bool) -> &mut Self {
    //     self.select = select;
    //     self
    // }
    // pub fn class(&mut self, class: &str) -> &mut Self {
    //     self.class = class.into();
    //     self
    // }
}

#[derive(EnumAsInner)]
pub enum ToolbarItem {
    Normal(ToolbarItemNormal),
    Html(ToolbarItemHtml),
    ToggleGroup(ToolbarToggleGroup),
}

pub struct ToolbarItemNormal {
    name: String,
    class: String,
    icon: String,
    // toggle_group: Option<String>,
}
pub struct ToolbarItemHtml {
    html: String,
}
pub struct ToolbarToggleGroup {
    name: String,
    class: String,
    icon: String,
    toggle_group: String,
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
    fn new_toggle_group(name: &str, icon: &str, class: &str, toggle_group: &str) -> Self {
        Self::ToggleGroup(ToolbarToggleGroup {
            name: name.into(),
            icon: icon.into(),
            class: class.into(),
            toggle_group: toggle_group.into(),
        })
    }
}
