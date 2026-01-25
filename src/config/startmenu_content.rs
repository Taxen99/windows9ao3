use rand::Rng;

use crate::config::{Action, Config, emit_div};

pub struct StartmenuContent {
    groups: Vec<StartmenuGroup>,
}

impl StartmenuContent {
    pub fn new() -> Self {
        Self { groups: Vec::new() }
    }
    pub fn group(
        &mut self,
        func: impl Fn(&mut StartmenuGroup) -> &mut StartmenuGroup,
    ) -> &mut Self {
        self.groups.push(StartmenuGroup::new());
        func(self.groups.last_mut().unwrap());
        self
    }

    pub fn build(&self, html: &mut String, css: &mut String, config: &Config) {
        emit_div(html, "smc", |html| {
            for groupp in &self.groups {
                emit_div(html, "smc-group", |html| {
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
                            StartmenuItem::Normal(item) => {
                                emit_div(html, "smc-item-outer", |html| {
                                    emit_div(html, &format!("smc-item {}", item.class), |html| {
                                        html.push_str(&format!(
                                            r##"
                                                <img src="{}" />
                                                <p>{}</p>
                                            "##,
                                            item.icon, item.name
                                        ));
                                    });
                                    // emit_div(html, "smc-item smc-single-click", |_| {});
                                });
                            }
                            StartmenuItem::Html(item) => {
                                html.push_str(&item.html);
                            }
                        }
                    }
                });
            }
        });
    }
}

pub struct StartmenuGroup {
    items: Vec<StartmenuItem>,
}

impl StartmenuGroup {
    fn new() -> Self {
        Self { items: Vec::new() }
    }
    pub fn item(&mut self, name: &str, icon: &str, class: &str) -> &mut Self {
        self.items.push(StartmenuItem::new(name, icon, class));
        self
    }
    pub fn item_html(&mut self, mut func: impl FnMut(&mut String)) -> &mut Self {
        let mut html = String::new();
        func(&mut html);
        self.items.push(StartmenuItem::new_html(html));
        self
    }
}

pub enum StartmenuItem {
    Normal(StartmenuItemNormal),
    Html(StartmenuItemHtml),
}

pub struct StartmenuItemNormal {
    name: String,
    class: String,
    icon: String,
}
pub struct StartmenuItemHtml {
    html: String,
}

impl StartmenuItem {
    fn new(name: &str, icon: &str, class: &str) -> Self {
        Self::Normal(StartmenuItemNormal {
            name: name.into(),
            icon: icon.into(),
            class: class.into(),
        })
    }
    fn new_html(html: String) -> Self {
        Self::Html(StartmenuItemHtml { html: html })
    }
}
