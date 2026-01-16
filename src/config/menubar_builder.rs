use rand::Rng;

use crate::config::{Action, Config, emit_div};

pub struct MenubarBuilder {
    items: Vec<MenubarItem>,
    is_short: bool,
}

impl MenubarBuilder {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            is_short: false,
        }
    }
    pub fn item(
        &mut self,
        name: &str,
        func: impl Fn(&mut MenubarItem) -> &mut MenubarItem,
    ) -> &mut Self {
        self.items.push(MenubarItem::new(name));
        func(self.items.last_mut().unwrap());
        self
    }
    pub fn short(&mut self, is_short: bool) -> &mut Self {
        self.is_short = is_short;
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
        let classlist = if self.is_short {
            "menubar menubar-short"
        } else {
            "menubar border-style-light-1"
        };
        emit_div(html, classlist, |html| {
            for item in &self.items {
                emit_div(html, "menubar-item", |html| {
                    html.push_str(r##"<div class="menubar-item-state"></div>"##);
                    html.push_str(&format!(r##"<p>{}</p>"##, item.name));
                    emit_div(html, "mb-submenu border-style-light-1", |html| {
                        for (i, group) in item.sub_item_groups.iter().enumerate() {
                            for sub_item in &group.sub_items {
                                let mut classlist = String::from("mb-submenu-item");
                                if sub_item.disabled {
                                    classlist.push_str(" mb-disabled");
                                }
                                // if let Some(id) = sub_item.id {
                                //     classlist.push_str(&format!(" mb-submenu-item-{}", id));
                                // }
                                let id = sub_item.id.unwrap_or_else(|| rand::rng().random());
                                let id_class = format!("mb-submenu-item-{}", id);
                                classlist.push_str(&format!(" {id_class}"));
                                match &sub_item.kind {
                                    SubItemKind::Dummy => {
                                        html.push_str(&format!(
                                            r##"<div class="{1}">
                                                <p>{0}</p>
                                            </div>"##,
                                            sub_item.name, classlist
                                        ));
                                    }
                                    SubItemKind::Toggle => {
                                        html.push_str(&format!(
                                            r##"<details class="{1}">
                                                <summary><p>{0}</p></summary>
                                            </details>"##,
                                            sub_item.name, classlist
                                        ));
                                    }
                                    SubItemKind::Action(action) => {
                                        html.push_str(&format!(
                                            r##"<div class="{1}">
                                                <p>{0}</p>
                                            </div>"##,
                                            sub_item.name, classlist
                                        ));
                                        config.emit_action(
                                            css,
                                            action,
                                            &format!(".{}:active", id_class),
                                        );
                                    }
                                }
                            }
                            // not last!
                            if i != item.sub_item_groups.len() - 1 {
                                html.push_str(r##"<div class="mb-submenu-separator"></div>"##);
                            }
                        }
                    });
                });
            }
        });
    }
}

pub struct MenubarItem {
    name: String,
    sub_item_groups: Vec<SubItemGroup>,
}

impl MenubarItem {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            sub_item_groups: Vec::new(),
        }
    }
    pub fn group(&mut self, func: impl Fn(&mut SubItemGroup) -> &mut SubItemGroup) -> &mut Self {
        self.sub_item_groups.push(SubItemGroup::new());
        func(self.sub_item_groups.last_mut().unwrap());
        self
    }
}

pub struct SubItemGroup {
    sub_items: Vec<SubItem>,
}

impl SubItemGroup {
    fn new() -> Self {
        Self {
            sub_items: Vec::new(),
        }
    }
    pub fn sub(&mut self, name: &str, func: impl Fn(&mut SubItem) -> &mut SubItem) -> &mut Self {
        self.sub_items.push(SubItem::new(name, false));
        func(self.sub_items.last_mut().unwrap());
        self
    }
    pub fn sub_disabled(&mut self, name: &str) -> &mut Self {
        self.sub_items.push(SubItem::new(name, true));
        self
    }
}

pub struct SubItem {
    name: String,
    kind: SubItemKind,
    id: Option<u64>,
    disabled: bool,
}

impl SubItem {
    fn new(name: &str, disabled: bool) -> Self {
        Self {
            name: name.to_owned(),
            kind: SubItemKind::Dummy,
            id: None,
            disabled,
        }
    }
    pub fn dummy(&mut self) -> &mut Self {
        self.kind = SubItemKind::Dummy;
        self
    }
    pub fn html_toggle(&mut self) -> &mut Self {
        self.kind = SubItemKind::Toggle;
        self
    }
    pub fn action(&mut self, action: Action) -> &mut Self {
        self.kind = SubItemKind::Action(action);
        self
    }
    pub fn id(&mut self, id: u64) -> &mut Self {
        self.id = Some(id);
        self
    }
}

pub enum SubItemKind {
    Dummy,
    Toggle,
    Action(Action),
}
