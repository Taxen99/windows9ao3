use crate::config::emit_div;

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

    pub fn build(&self, html: &mut String, css: &mut String) {
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
            "menubar"
        };
        emit_div(html, classlist, |html| {
            for item in &self.items {
                emit_div(html, "menubar-item", |html| {
                    html.push_str(r##"<div class="menubar-item-state"></div>"##);
                    html.push_str(&format!(r##"<p>{}</p>"##, item.name));
                    emit_div(html, "mb-submenu border-style-light-1", |html| {
                        for (i, group) in item.sub_item_groups.iter().enumerate() {
                            for sub_item in &group.sub_items {
                                html.push_str(&format!(
                                    r##"<div class="mb-submenu-item">
                                        <p>{}</p>
                                    </div>"##,
                                    sub_item.name
                                ));
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
    pub fn item(&mut self, name: &str, action: ()) -> &mut Self {
        self.sub_items.push(SubItem {
            name: name.to_owned(),
            action,
        });
        self
    }
}

pub struct SubItem {
    name: String,
    action: (),
}
