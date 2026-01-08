use std::collections::HashMap;
use std::fs;
use std::hash::{DefaultHasher, Hash, Hasher};

use enum_as_inner::EnumAsInner;
use serde::{Deserialize, Serialize};

use crate::css_var_remove::css_var_remove;

#[derive(Serialize, Deserialize)]
pub struct App {
    pub name: String,
    pub icon: String,
    pub add_to_desktop: Option<(u8, u8)>,
    pub content: String,
}

impl App {}

#[derive(Serialize, Deserialize)]
pub struct FileSystem {
    pub root: Folder,
}

#[derive(Serialize, Deserialize, EnumAsInner)]
pub enum FsEntry {
    File(File),
    Folder(Folder),
}
impl FsEntry {
    pub fn offset(&self) -> Option<(u8, u8)> {
        match self {
            FsEntry::File(file) => file.offset,
            FsEntry::Folder(folder) => folder.offset,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Folder {
    pub content: HashMap<String, FsEntry>,
    pub offset: Option<(u8, u8)>,
}
#[derive(Serialize, Deserialize, PartialEq, Eq)]
pub enum FileKind {
    App,
}
#[derive(Serialize, Deserialize)]
pub struct File {
    pub kind: FileKind,
    pub link: String,
    pub offset: Option<(u8, u8)>,
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub apps: Vec<App>,
    pub fs: FileSystem,
}

trait StrExt {
    fn hashed(self) -> u64;
}
impl StrExt for &str {
    // TODO: quick and dirty solution!
    fn hashed(self) -> u64 {
        let mut s = DefaultHasher::new();
        self.hash(&mut s);
        s.finish()
    }
}

#[derive(Debug, Default)]
pub struct BuildResult {
    pub html: String,
    pub css: String,
    pub ao3_html: String,
    pub ao3_css: String,
}
impl BuildResult {
    fn from_html_css(html_in: String, css_in: String) -> BuildResult {
        let html = format!(
            r##"
            <!DOCTYPE html>
            <html lang="en">
            <head>
                <meta charset="UTF-8">
                <meta name="viewport" content="width=device-width, initial-scale=1.0">
                <title>Document</title>
                <link rel="stylesheet" href="style.css">
            </head>
            <body>
                {html_in}
            </body>
            </html>
            "##
        );
        let ao3_html = html_in;
        let css = css_in;
        let ao3_css = css_var_remove(&css);
        Self {
            html,
            css,
            ao3_html,
            ao3_css,
        }
    }
}

impl Config {
    pub fn build(mut self) -> BuildResult {
        self.app_apps_to_desktop();

        let mut css = String::new();
        let mut html = String::new();
        css.push_str(&load_css());
        emit_div(&mut html, "main", |html| {
            emit_div(html, "desktop", |html| {
                if let Some(desktop) = self.fs.root.content.get("desktop") {
                    let desktop = desktop.as_folder().expect("desktop must be folder");
                    for (name, entry) in &desktop.content {
                        let offset = entry.offset().unwrap_or_default();
                        html.push_str(&format!(
                            r##"
                                <div class="desktop-item desktop-item-{}">
                                    <div class="desktop-icon"></div>
                                    <p class="desktop-icon-name">{}</p>
                                    <div class="desktop-item-double-clicker"></div>
                                </div>
                                <div class="desktop-item-animator">
                                    <div class="desktop-item-animator-helper"></div>
                                </div>
                                "##,
                            name.hashed(),
                            name
                        ));
                        css.push_str(&format!(
                            r##"
                            .desktop-item-{0} .desktop-icon {{
                                        background: url("{1}");
                                        background-size: cover;
                                    }}
                                    .desktop-item-{0} {{
                                        left: {2}px;
                                        top: {3}px;
                                    }}
                            "##,
                            name.hashed(),
                            self.icon_of(entry),
                            offset.0 * 54,
                            offset.1 * 64,
                        ));
                        if let Some(file) = entry.as_file()
                            && file.kind == FileKind::App
                        {
                            css.push_str(&format!(
                                    r##"
                                    
                                    .desktop:has(.desktop-item-{0} + .desktop-item-animator .desktop-item-animator-helper:hover) ~ .window-{1} {{
                                        top: 30px;
                                        left: 30px;
                                        transition: top 0s linear 0s, left 0s linear 0s;
                                    }}
                                    "##,
                                    name.hashed(),
                                    file.link.hashed()
                                ));
                        }
                    }
                }
                for w in self.apps.iter().filter(|x| x.add_to_desktop.is_some()) {
                    let pos = w.add_to_desktop.unwrap();
                }
            });
            for w in &self.apps {
                html.push_str(&format!(
                            r##"
                            <div class="window window-{0}">
                                <div class="window-titlebar">
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
                                    <div class="window-name">
                                        <p>{1}</p>
                                    </div>
                                    <div class="window-exiter"><p>X</p></div>
                                </div>
                                <div class="window-content">
                                    <div class="content-inner">
                                        <p>{2}</p>
                                    </div>
                                </div>
                            </div>
                            "##,
                            w.name.hashed(),
                            w.name,
                            w.content
                        ));
                css.push_str(&format!(
                    r##"
                            .window-{0} .mover {{
                                background: url("{1}");
                                background-size: cover;
                            }}
                            "##,
                    w.name.hashed(),
                    w.icon,
                ));
            }
        });
        BuildResult::from_html_css(html, css)
    }
    fn app_apps_to_desktop(&mut self) {
        let mut app_files = HashMap::new();
        for app in &self.apps {
            if let Some(desktop_pos) = app.add_to_desktop {
                app_files.insert(
                    app.name.clone(),
                    File {
                        kind: FileKind::App,
                        link: app.name.clone(),
                        offset: Some(desktop_pos),
                    },
                );
            }
        }

        if let Some(desktop) = self.fs.root.content.get_mut("desktop") {
            let desktop = desktop.as_folder_mut().expect("desktop must be folder");
            for (k, v) in app_files.drain() {
                if desktop.content.insert(k, FsEntry::File(v)).is_some() {
                    panic!("publicate file name");
                }
            }
        }
    }
    fn app(&self, name: &str) -> &App {
        self.apps
            .iter()
            .find(|x| x.name == name)
            .expect(&format!("app '{name}' does not exist"))
    }
    fn icon_of(&self, entry: &FsEntry) -> String {
        match entry {
            FsEntry::File(file) => match file.kind {
                FileKind::App => self.app(&file.link).icon.clone(),
            },
            FsEntry::Folder(_folder) => {
                // TODO: what if this resource disappears?
                "https://win98icons.alexmeub.com/icons/png/directory_closed-4.png".to_owned()
            }
        }
    }
}
fn emit_div(s: &mut String, class: &str, mut cb: impl FnMut(&mut String)) {
    s.push_str(&format!(r##"<div class="{class}">"##));
    cb(s);
    s.push_str(&format!(r##"</div>"##));
}
fn load_css() -> String {
    let mut res = String::new();
    for e in fs::read_dir("src/bin/res/css").unwrap() {
        let e = e.unwrap();
        assert!(e.path().extension().unwrap() == "css");
        res.push_str(std::fs::read_to_string(e.path()).unwrap().as_str());
        res.push('\n');
    }
    res
}
