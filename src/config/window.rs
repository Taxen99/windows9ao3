pub struct Window {
    pub id: u64,
    pub name: String,
    pub icon: Option<String>,
    pub extra_classes: Option<String>,
    pub should_appear_in_taskbar: bool,
    pub exitable: bool,
    pub resizable: bool,
    pub inject_outside: Option<String>,
    pub custom_title: Option<String>,
}

impl Window {
    pub fn new(id: u64, name: &str) -> Self {
        Self {
            id,
            name: name.into(),
            icon: None,
            extra_classes: None,
            should_appear_in_taskbar: true,
            exitable: true,
            resizable: true,
            inject_outside: None,
            custom_title: None,
        }
    }
    pub fn icon(mut self, icon: &str) -> Self {
        self.icon = Some(icon.into());
        self
    }
    pub fn extra_classes(mut self, extra_classes: &str) -> Self {
        self.extra_classes = Some(extra_classes.into());
        self
    }
    pub fn should_appear_in_taskbar(mut self, should_appear_in_taskbar: bool) -> Self {
        self.should_appear_in_taskbar = should_appear_in_taskbar;
        self
    }
    pub fn exitable(mut self, exitable: bool) -> Self {
        self.exitable = exitable;
        self
    }
    pub fn resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
        self
    }
    pub fn inject_outside(mut self, inject_outside: &str) -> Self {
        self.inject_outside = Some(inject_outside.into());
        self
    }
    pub fn custom_title(mut self, mut func: impl FnMut(&mut String)) -> Self {
        let mut html = String::new();
        func(&mut html);
        self.custom_title = Some(html);
        self
    }
    pub fn build(
        self,
        html: &mut String,
        css: &mut String,
        config: &Config,
        mut func: impl FnMut(&mut String, &mut String),
    ) {
        if self.should_appear_in_taskbar {
            if !config.state.borrow_mut().windows.insert((
                self.id,
                self.name.clone(),
                self.icon
                    .as_deref()
                    .expect("if can appear in taskbar, then should have icon, probably")
                    .into(),
            )) {
                panic!("the fook are you doing?");
            }
        }
        emit_div(
            html,
            &format!(
                "window window-{0} {1}",
                self.id,
                self.extra_classes.unwrap_or("".into())
            ),
            |html| {
                html.push_str(self.inject_outside.as_deref().unwrap_or(""));
                emit_div(html, "window-inner", |html| {
                    if self.resizable {
                        emit_div(html, "wra wra-hor wra-hor-left", |html| {
                            emit_div(html, "wr wr-left", |_| {});
                            emit_div(html, "wr wr-right", |_| {});
                        });
                        emit_div(html, "wra wra-hor wra-hor-right", |html| {
                            emit_div(html, "wr wr-left", |_| {});
                            emit_div(html, "wr wr-right", |_| {});
                        });
                        emit_div(html, "wra wra-ver wra-ver-up", |html| {
                            emit_div(html, "wr wr-up", |_| {});
                            emit_div(html, "wr wr-down", |_| {});
                        });
                        emit_div(html, "wra wra-ver wra-ver-down", |html| {
                            emit_div(html, "wr wr-up", |_| {});
                            emit_div(html, "wr wr-down", |_| {});
                        });
                    }
                    emit_div(html, "window-titlebar", |html| {
                        emit_div(html, "mover-anchors", |html| {
                            for i in 0..Config::MOVER_ANCHORS_COUNT {
                                emit_div(html, &format!("mover-anchor mover-anchor-{i}"), |_| ());
                            }
                        });
                        html.push_str(r##"
                            <div class="mover">
                                <div class="mover-hand mover-hand-Q mover-hand-up mover-hand-left "></div>
                                <div class="mover-hand mover-hand-W mover-hand-up "></div>
                                <div class="mover-hand mover-hand-E mover-hand-up mover-hand-right "></div>
                                <div class="mover-hand mover-hand-A mover-hand-left "></div>
                                <div class="mover-hand mover-hand-D mover-hand-right "></div>
                                <div class="mover-hand mover-hand-Z mover-hand-down mover-hand-left "></div>
                                <div class="mover-hand mover-hand-X mover-hand-down  "></div>
                                <div class="mover-hand mover-hand-C mover-hand-down mover-hand-right "></div>
                            </div>
                        "##);
                        if self.icon.is_some() {
                            emit_div(html, "window-icon", |_| {});
                        }
                        emit_div(html, "window-name", |html| {
                            if let Some(title) = &self.custom_title {
                                html.push_str(title);
                            } else {
                                emit_p(html, "", &self.name);
                            }
                        });
                        if self.exitable {
                            emit_div(html, "window-exiter", |_| {});
                        }
                    });
                    emit_div(html, "window-content", |html| {
                        func(html, css);
                    });
                });
            },
        );
        if self.exitable {
            config.emit_action(
                css,
                &Action::Close(self.id),
                &format!(".window-{0} .window-exiter:active", self.id),
            );
        }
        if let Some(icon) = &self.icon {
            css.push_str(&format!(
                r##"
                .window-{0} .window-icon {{
                    background: url("{1}");
                    background-size: cover;
                }}
                "##,
                self.id, icon,
            ));
        }
    }
}

use crate::config::{Action, Config, emit_div, emit_img, emit_p};

#[derive(Debug, Default, Clone)]
pub struct Dialog {
    pub id: u64,
    pub name: String,
    pub icon: Option<String>,
    pub symbol: DialogueSymbol,
    pub kind: DialogueKind,
    pub content: String,
    pub trigger: Option<String>,
    pub long: bool,
    pub extra_classes: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum DialogueSymbol {
    #[default]
    Error,
    Warning,
    Custom(String),
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DialogueKind {
    #[default]
    Ok,
    YesNo,
}

impl Dialog {
    pub fn new(
        id: u64,
        name: &str,
        symbol: DialogueSymbol,
        kind: DialogueKind,
        content: &str,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            icon: None,
            symbol,
            kind,
            content: content.into(),
            trigger: None,
            long: false,
            extra_classes: String::new(),
        }
    }
    pub fn icon(mut self, icon: &str) -> Self {
        self.icon = Some(icon.into());
        self
    }
    pub fn trigger(mut self, trigger: &str) -> Self {
        self.trigger = Some(trigger.into());
        self
    }
    pub fn long(mut self, long: bool) -> Self {
        self.long = long;
        self
    }
    pub fn extra_classes(mut self, extra_classes: &str) -> Self {
        self.extra_classes = extra_classes.into();
        self
    }
    pub fn build(self, html: &mut String, css: &mut String, config: &Config) {
        let mut w = Window::new(self.id, &self.name);
        w.icon = self.icon.clone();
        w = w.exitable(false);
        w = w.should_appear_in_taskbar(false);
        let size_class = self.long.then_some("dialogue-long").unwrap_or("");
        w = w.extra_classes(&format!("dialogue {size_class} {}", self.extra_classes));
        w = w.resizable(false);
        w = w.inject_outside(&format!(
            r##"
            <div class="dialogue-ding">
                <audio controls="controls" src="@audio:ding.wav">
            </div>
            "##
        ));
        if let Some(trigger) = self.trigger.as_deref() {
            config.emit_action(css, &Action::OpenDialog(self.id), trigger);
        }
        w.build(html, css, config, |html, css| {
            emit_div(html, "window-main", |html| {
                emit_div(html, "dialogue-main", |html| {
                    emit_div(html, "dialogue-upper", |html| {
                        let symbol_src = match &self.symbol {
                            DialogueSymbol::Error => "@icon:dialogue-err",
                            DialogueSymbol::Warning => "@icon:dialogue-warn",
                            DialogueSymbol::Custom(x) => x
                        };
                        emit_img(html, "dialogue-symbol", symbol_src);
                        emit_div(html, "dialogue-view", |html| {
                            emit_p(html, "", &self.content);
                        });
                    });
                    emit_div(html, "dialogue-lower", |html| match self.kind {
                        DialogueKind::Ok => {
                            emit_div(
                                html,
                                "dialogue-button dialogue-button-focus border-style-asymmetric-1 dialogue-ok",
                                |html| emit_p(html, "", "OK"),
                            );
                        }
                        DialogueKind::YesNo => {
                            emit_div(
                                html,
                                "dialogue-button dialogue-button-focus border-style-asymmetric-1 dialogue-yes",
                                |html| emit_p(html, "", "Yes"),
                            );
                            emit_div(
                                html,
                                "dialogue-button border-style-asymmetric-1 dialogue-no",
                                |html| emit_p(html, "", "No"),
                            );
                        }
                    });
                    config.emit_action(css, &Action::Close(self.id), &format!(".window-{} .dialogue-button:active", self.id));
                });
            });
        });
    }
}
