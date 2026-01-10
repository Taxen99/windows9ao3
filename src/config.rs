use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::{Path, PathBuf};
use std::{fs, path};

use enum_as_inner::EnumAsInner;
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::css_var_remove::css_var_remove;

#[derive(Debug, Serialize, Deserialize)]
pub struct App {
    pub name: String,
    pub icon: String,
    pub add_to_desktop: Option<(u8, u8)>,
    pub content: String,
}

impl App {}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileSystem {
    pub root: FsEntry,
}

#[derive(Debug, Serialize, Deserialize, EnumAsInner)]
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

#[derive(Debug, Serialize, Deserialize)]
pub struct Folder {
    pub children: HashMap<String, FsEntry>,
    pub offset: Option<(u8, u8)>,
}
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum FileKind {
    App,
    Shortcut,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct File {
    pub kind: FileKind,
    pub link: String,
    pub offset: Option<(u8, u8)>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub apps: Vec<App>,
    pub fs: FileSystem,
}

trait HashedExt {
    fn hashed(self) -> u64;
}
impl<T> HashedExt for &T
where
    T: Hash,
{
    // TODO: quick and dirty solution! this is really scuffed
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
    const FILE_EXPLORER_ID: u64 = 1;
    pub fn build(mut self) -> BuildResult {
        self.app_apps_to_desktop();

        let mut css = String::new();
        let mut html = String::new();
        css.push_str(&load_css());
        emit_div(&mut html, "main", |html| {
            emit_div(html, "desktop", |html| {
                if let Some(desktop) = self.fs.root.as_folder().unwrap().children.get("desktop") {
                    let _ = desktop.as_folder().expect("desktop must be folder");
                    self.emit_file_view_content(html, &mut css, Path::new("/desktop"));
                }
            });
            for w in &self.apps {
                self.emit_window(
                    html,
                    &mut css,
                    w.name.hashed(),
                    &w.name,
                    &w.icon,
                    |html, css| {
                        html.push_str(&format!(
                            r##"<div class="content-inner">
                                <p>{}</p>
                            </div>"##,
                            w.content
                        ));
                    },
                );
            }
            self.emit_fe_window(html, &mut css);
        });
        BuildResult::from_html_css(html, css)
    }
    fn emit_fe_window(&self, html: &mut String, css: &mut String) {
        self.emit_window(
            html,
            css,
            Self::FILE_EXPLORER_ID,
            "File Explorer",
            "https://win98icons.alexmeub.com/icons/png/computer_explorer-5.png",
            |html, css| {
                html.push_str(&format!(r##"
                <div class="window-header">
                    <div class="topbar menubar border-style-light-1">
                        <div class="menubar-item">
                            <div class="menubar-item-state"></div>
                            <p>File</p>
                            <div class="mb-submenu border-style-light-1">
                                <div class="mb-submenu-item">
                                    <p>Open</p>
                                </div>
                                <div class="mb-submenu-item">
                                    <p>Close</p>
                                </div>
                                <div class="mb-submenu-item">
                                    <p>Redact</p>
                                </div>
                            </div>
                        </div>
                        <div class="menubar-item"><div class="menubar-item-state"></div><p>Edit</p></div>
                        <div class="menubar-item"><div class="menubar-item-state"></div><p>View</p></div>
                        <div class="menubar-item"><div class="menubar-item-state"></div><p>Go</p></div>
                        <div class="menubar-item"><div class="menubar-item-state"></div><p>Favorites</p></div>
                        <div class="menubar-item"><div class="menubar-item-state"></div><p>Help</p></div>
                    </div>
                    <div class="topbar fe-address-bar border-style-light-1">
                        <p>Address</p>
                        <p class="border-style-dark-1">My Computer</p>
                    </div>
                </div>
                "##));
                //
                emit_div(html, "window-main", |html| {
                    emit_div(html, "fe-main", |html| {
                        emit_div(html, "fe-main", |html| {
                            emit_div(html, "fe-sideview border-style-light-1", |html| {
                                html.push_str(
                                    r##"<div class="fe-sideview-header">
                                                    <p>All Folders</p>
                                                </div>"##,
                                );
                                emit_div(html, "fe-sideview-view", |html| {
                                    fn emit_folder(html: &mut String, css: &mut String, folder: &Folder, path: PathBuf) {
                                        let folder_hash = path.hashed();
                                        let mut sub_folder_count = 0;
                                        emit_div(html, &format!("fe-svv-child fe-svv-child-{}", folder_hash), |html| {
                                            for (name, entry) in &folder.children {
                                                if let FsEntry::Folder(sub_folder) = &entry {
                                                    sub_folder_count += 1;
                                                    emit_div(html, "fe-svv-item", |html| {
                                                        html.push_str(&format!(
                                                            r##"
                                                            <div class="fe-svvi-group">
                                                                <div class="fe-svvi-expander"></div>
                                                                <p class="fe-svvi-name">{}</p>
                                                            </div>"##,
                                                            name
                                                        ));
                                                        let mut sub_path = path.clone();
                                                        sub_path.push(name);
                                                        emit_folder(html, css, sub_folder, sub_path);
                                                    });
                                                }
                                            }
                                        });
                                        css.push_str(&format!(r##"
                                        .fe-svv-child-{}::before {{
                                            height: {}px;
                                        }}
                                        "##, folder_hash, 14 * sub_folder_count - 3));
                                        dbg!(folder.children.len(), path);
                                    }
                                    emit_folder(html, css, &self.fs.root.as_folder().unwrap(), PathBuf::from("/"));
                                });
                            });
                            emit_div(html, "fe-view-anchor", |html| {
                                fn emit_folder(config: &Config, html: &mut String, css: &mut String, folder: &Folder, path: PathBuf) {
                                    let folder_hash = path.hashed();
                                    emit_div(html, &format!("fe-view border-style-dark-1 fe-view-{}", folder_hash), |html| {
                                        config.emit_file_view_content(html, css, &path);
                                    });
                                    for (name, entry) in &folder.children {
                                        if let FsEntry::Folder(sub_folder) = &entry {
                                            let mut sub_path = path.clone();
                                            sub_path.push(name);
                                            emit_folder(config, html, css, sub_folder, sub_path);
                                        }
                                    }
                                }
                                emit_folder(self, html, css, &self.fs.root.as_folder().unwrap(), PathBuf::from("/"));
                            });
                        });
                    });
                });
            },
        );
    }
    fn emit_window(
        &self,
        html: &mut String,
        css: &mut String,
        id: u64,
        name: &str,
        icon: &str,
        mut cb: impl FnMut(&mut String, &mut String),
    ) {
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
                    <div class="window-exiter"></div>
                </div>
                <div class="window-content">
            "##,
            id,
            name,
        ));
        cb(html, css);
        html.push_str(
            r##"
                </div>
            </div>
        "##,
        );
        css.push_str(&format!(
            r##"
                            .window-{0} .mover {{
                                background: url("{1}");
                                background-size: cover;
                            }}
                            "##,
            id, icon,
        ));
    }
    fn emit_file_view_content(
        &self,
        html: &mut String,
        css: &mut String,
        // id: u64,
        // folder: &Folder,
        path: &path::Path,
        // mut cb: impl FnMut(&mut String, &mut String),
    ) {
        // dbg!(path);
        let folder = self.fs_entry(path).unwrap().as_folder().unwrap();
        for (name, entry) in &folder.children {
            // dbg!(name);
            let mut entry_path = path.to_owned();
            entry_path.push(name);
            let unique_hash: u64 = rand::rng().random();
            // let unique_hash =
            //     format!("{}___{}", entry_path.to_str().unwrap(), some_random_numer).hashed();
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
                unique_hash, name
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
                unique_hash,
                self.icon_of(entry),
                offset.0 * 54,
                offset.1 * 64,
            ));
            self.emit_fe_interaction(css, entry, &entry_path, unique_hash);
        }
    }
    fn emit_fe_interaction(
        &self,
        css: &mut String,
        entry: &FsEntry,
        entry_path: &Path,
        file_unique_hash: u64,
    ) {
        if let Some(file) = entry.as_file() {
            match file.kind {
                FileKind::App => {
                    css.push_str(&format!(
                            r##"
                            
                            .main:has(.desktop-item-{0} + .desktop-item-animator .desktop-item-animator-helper:hover) .window-{1} {{
                                top: 30px;
                                left: 30px;
                                transition: top 0s linear 0s, left 0s linear 0s;
                            }}
                            "##,
                            file_unique_hash,
                            file.link.hashed()
                        ));
                }
                FileKind::Shortcut => {
                    let new_path = Path::new(&file.link);
                    let new_entry = self.fs_entry(new_path).unwrap();
                    self.emit_fe_interaction(css, new_entry, new_path, file_unique_hash);
                }
            }
        }
        if let Some(_sub_folder) = entry.as_folder() {
            css.push_str(&format!(
                    r##"
                    
                    .main:has(.desktop-item-{0} + .desktop-item-animator .desktop-item-animator-helper:hover) .window-{1} {{
                        top: 30px;
                        left: 30px;
                        transition: top 0s linear 0s, left 0s linear 0s;
                    }}
                    .main:has(.desktop-item-{0} + .desktop-item-animator .desktop-item-animator-helper:hover) .fe-view-{2} {{
                        left: 0px;
                        transition: left 0s linear;
                    }}
                    .main:has(.desktop-item-{0} + .desktop-item-animator .desktop-item-animator-helper:hover) .fe-view:not(.fe-view-{2}) {{
                        left: -20000.001px;
                        transition: left 0s linear;
                    }}
                    "##,
                    file_unique_hash,
                    Self::FILE_EXPLORER_ID,
                    entry_path.hashed(),
                ));
        }
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

        if let Some(desktop) = self
            .fs
            .root
            .as_folder_mut()
            .unwrap()
            .children
            .get_mut("desktop")
        {
            let desktop = desktop.as_folder_mut().expect("desktop must be folder");
            for (k, v) in app_files.drain() {
                if desktop.children.insert(k, FsEntry::File(v)).is_some() {
                    panic!("publicate file name");
                }
            }
        }
    }
    fn fs_entry(&self, path: &path::Path) -> Option<&FsEntry> {
        if path == "/" {
            return Some(&self.fs.root);
        }
        let parts: Vec<_> = path.to_str().unwrap().split("/").skip(1).collect();
        let mut entry: &FsEntry = &self.fs.root;
        for part in parts {
            entry = &entry.as_folder()?.children[part];
        }
        Some(entry)
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
                FileKind::Shortcut => self.icon_of(self.fs_entry(Path::new(&file.link)).unwrap()),
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
    visit_dir(path::Path::new("src/bin/res/css"), &mut |path| {
        assert!(path.extension().unwrap() == "css");
        res.push_str(std::fs::read_to_string(path).unwrap().as_str());
        res.push('\n');
    });
    res
}

fn visit_dir(dir: &path::Path, cb: &mut dyn FnMut(&path::Path)) {
    for e in fs::read_dir(dir).unwrap() {
        let e = e.unwrap();
        let path = e.path();
        if path.is_dir() {
            visit_dir(&path, cb);
        } else {
            cb(&path);
        }
    }
}
