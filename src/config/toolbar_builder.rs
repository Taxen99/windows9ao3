use rand::Rng;

use crate::config::{Action, Config, emit_div};

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

    pub fn build(&self, html: &mut String, css: &mut String, config: &Config) {
        /*
        <div class="menubar menubar-short">
                        <div class="menubar-item">
                            <div class="menubar-item-state"></div>
                            <p>File</p>
                        </div>
                        <div class="menubar-item"><div class="menubar-item-state"></div><p>Edit</p></div>
                        <div class="menubar-item"><div class="menubar-item-state"></div><p>Search</p></div>
                        <div class="menubar-item"><div class="menubar-item-state"></div><p>Help</p></div>
                    </div>
         */
        // let classlist = if self.is_short {
        //     "menubar menubar-short"
        // } else {
        //     "menubar"
        // };
        emit_div(html, "toolbar border-style-light-1", |html| {
            for groupp in &self.groups {
                emit_div(html, "toolbar-group", |html| {
                    for item in groupp.items.iter() {
                        // let mut classlist = String::from("mb-submenu-item");
                        // if sub_item.disabled {
                        //     classlist.push_str(" mb-disabled");
                        // }
                        // if let Some(id) = sub_item.id {
                        //     classlist.push_str(&format!(" mb-submenu-item-{}", id));
                        // }
                        // let id = sub_item.id.unwrap_or_else(|| rand::rng().random());
                        // let id_class = format!("mb-submenu-item-{}", id);
                        // classlist.push_str(&format!(" {id_class}"));
                        match item {
                            ToolbarItem::Normal(item) => {
                                emit_div(html, "toolbar-item-outer", |html| {
                                    emit_div(
                                        html,
                                        &format!("toolbar-item {}", item.class),
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
                                });
                            }
                            ToolbarItem::Html(item) => {
                                html.push_str(&item.html);
                            }
                        }
                    }
                });
            }
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
