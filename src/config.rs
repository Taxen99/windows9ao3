use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::ffi::OsStr;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::{Path, PathBuf};
use std::{fs, path};

use enum_as_inner::EnumAsInner;
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::config::history::{History, HistoryItem};
use crate::config::internet_explorer::read_sites;
use crate::config::menubar_builder::MenubarBuilder;
use crate::config::startmenu_content::StartmenuContent;
use crate::config::toolbar_builder::ToolbarBuilder;
use crate::config::vertical_select::emit_vertical_select;
use crate::css_var_remove::css_var_remove;

mod history;
mod internet_explorer;
mod menubar_builder;
mod startmenu_content;
mod toolbar_builder;
mod vertical_select;

#[derive(Debug, Serialize, Deserialize)]
pub struct App {
    pub name: String,
    pub icon: String,
    pub add_to_desktop: Option<(u32, u32)>,
    pub content: String,
}

impl App {}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileSystem {
    pub root: FsEntry,
}

impl FileSystem {
    pub fn visit_all_files(&self, mut func: impl FnMut(&File, &Path)) {
        fn visit(f: &FsEntry, p: &Path, mut cb: &mut dyn FnMut(&File, &Path)) {
            match f {
                FsEntry::File(file) => cb(file, p),
                FsEntry::Folder(folder) => {
                    for (child_name, child_entry) in &folder.children {
                        let mut child_path = p.to_owned();
                        child_path.push(child_name);
                        visit(&child_entry, &child_path, cb);
                    }
                }
            };
        }
        visit(&self.root, Path::new("/"), &mut func);
    }
    pub fn visit_all_folders(&self, mut func: impl FnMut(&Folder, &Path)) {
        fn visit(f: &FsEntry, p: &Path, mut cb: &mut dyn FnMut(&Folder, &Path)) {
            match f {
                FsEntry::File(_) => (),
                FsEntry::Folder(folder) => {
                    cb(folder, p);
                    for (child_name, child_entry) in &folder.children {
                        let mut child_path = p.to_owned();
                        child_path.push(child_name);
                        visit(&child_entry, &child_path, cb);
                    }
                }
            };
        }
        visit(&self.root, Path::new("/"), &mut func);
    }
}

#[derive(Debug, Serialize, Deserialize, EnumAsInner)]
pub enum FsEntry {
    File(File),
    Folder(Folder),
}
impl FsEntry {
    pub fn offset(&self) -> Option<(u32, u32)> {
        match self {
            FsEntry::File(file) => file.offset,
            FsEntry::Folder(folder) => folder.offset,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Folder {
    pub children: BTreeMap<String, FsEntry>,
    pub offset: Option<(u32, u32)>,
}
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum FileKind {
    App,
    NativeApp,
    Shortcut,
    Text,
    Image,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct File {
    pub kind: FileKind,
    pub link: String,
    pub offset: Option<(u32, u32)>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub apps: Vec<App>,
    pub fs: FileSystem,
    // NOTE: building *should* be deterministic, but recalculating shit is boring and beta cuck behaviour. be the sigma. have the skibidi rizz.
    #[serde(skip)]
    pub state: RefCell<ConfigState>,
}

#[derive(Debug, Default)]
pub struct ConfigState {
    windows: HashSet<(u64, String, String)>,
}

pub enum Action {
    Close(u64),
    Open(u64),
    OpenFileExplorer(u64),
    OpenNotepad(u64),
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
    const INTERNET_EXPLORER_ID: u64 = 69;
    const DATE_TIME_PROPERTIES_ID: u64 = 420;
    const QUICK_LAUNCH_SUPPORTED_APPS: &[u64] =
        &[Self::FILE_EXPLORER_ID, Self::INTERNET_EXPLORER_ID];
    const INITIAL_TIME: (u32, u32) = (06, 34);
    // const NOTEPAD_ID: u64 = 2;
    // const NOTEPAD_FONT_ID: u64 = 3;
    pub fn build(mut self) -> BuildResult {
        self.app_apps_to_desktop();

        let mut css = String::new();
        let mut html = String::new();
        css.push_str(&load_css());
        emit_div(&mut html, "main", |html| {
            emit_div(html, "onload", |_| ());
            emit_div(html, "desktop", |html| {
                if let Some(desktop) = self.fs.root.as_folder().unwrap().children.get("desktop") {
                    let _ = desktop.as_folder().expect("desktop must be folder");
                    self.emit_file_view_content(html, &mut css, Path::new("/desktop"), false);
                }
            });
            emit_div(html, "windows-container", |html| {
                for w in &self.apps {
                    self.emit_window(
                        html,
                        &mut css,
                        w.name.hashed(),
                        &w.name,
                        &w.icon,
                        None,
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
                self.emit_np_window(html, &mut css);
                self.emit_qv_window(html, &mut css);
                self.emit_ie_window(html, &mut css);
                self.emit_dt_window(html, &mut css);
            });
            self.emit_taskbar(html, &mut css);
        });
        self.emit_action(
            &mut css,
            &Action::OpenFileExplorer(Path::new("/").hashed()),
            ".onload:hover",
        );
        BuildResult::from_html_css(html, css)
    }
    fn emit_time_advancers(&self, html: &mut String, css: &mut String) {
        emit_div(html, "minute-advancers", |html| {
            for i in 0..60 {
                emit_div(
                    html,
                    &format!("minute-advancer minute-advancer-{}", i),
                    |_| (),
                );
            }
        });
        emit_div(html, "hour-advancers", |html| {
            for i in 0..24 {
                emit_div(html, &format!("hour-advancer hour-advancer-{}", i), |_| ());
            }
        });

        for i in 0..24 {
            css.push_str(&format!(
                // NOTE: we must specify 'content' in the transition because something something transitions FUCK YOU (jkjk)
                r##"
.main:has(.hour-advancer-{0}:hover) .time-hour::before {{
            transition: content 0s;
            content: "{1:02}"
}}
.main:has(.hour-advancer-{0}:hover) .dt-hour-hand {{
    transition: 0s;
    transform: translate(-22.64%, -78.78%) rotateZ({2}deg)
}}
"##,
                i,
                (i + 1) % 24,
                -60 + ((i + 1) % 24) * 30
            ));
        }
        for i in 0..60 {
            let color = if i % 2 == 0 { "red" } else { "green" };
            css.push_str(&format!(
                r##"
.main:has(.minute-advancer-{0}:hover) .time-minute::before {{
            transition: content 0s;
            content: "{1:02}";
            background: {color};
}}
.main:has(.minute-advancer-{0}:hover) .dt-minute-hand {{
            transition: 0s;
            transform: translate(-22.64%, -78.78%) rotateZ({2}deg)
}}
"##,
                i,
                (i + 1) % 60,
                -60 + ((i + 1) % 60) * 6
            ));
        }
        css.push_str(&format!(
            // .main:has(.onload:hover) .time-minute::before {{
            //     transition: content 0s;
            //     content: "{0:02}";
            // }}
            // .main:has(.onload:hover) .time-hour::before {{
            //     transition: content 0s;
            //     content: "{1:02}";
            // }}
            r##"
            .main:has(.onload:hover) .minute-advancer:nth-child({}) {{
                transition: 0s;
                z-index: 2147483640;
                top: 0;
            }}
            .main:has(.onload:hover) .hour-advancer:nth-child({}) {{
                transition: 0s;
                z-index: 2147483640 !important;
                top: 0;
            }}
        "##,
            // Self::INITIAL_TIME.1,
            // Self::INITIAL_TIME.0,
            Self::INITIAL_TIME.1,
            Self::INITIAL_TIME.0
        ));
        css.push_str(
            r##"
            
        "##,
        );
    }
    fn emit_taskbar(&self, html: &mut String, css: &mut String) {
        self.emit_time_advancers(html, css);
        emit_div(html, "taskbar border-style-light-1", |html| {
            emit_div(html, "tb-item tb-start", |html| {
                emit_div(
                    html,
                    "tb-start-button tb-start-button-unactive border-style-asymmetric-1",
                    |html| {
                        emit_p(html, "tb-sb-p", "Start");
                    },
                );
                emit_div(html, "tb-start-button tb-start-button-active", |html| {
                    emit_div(html, "tb-start-button-active-inner", |html| {
                        emit_p(html, "tb-sb-p", "Start");
                    });
                    emit_div(html, "tb-start-menu border-style-dark-3", |html| {
                        emit_div(html, "tb-sm-banner", |html| {
                            html.push_str(r##"<img src="../res/icons/windows98-start.png" />"##);
                        });
                        emit_div(html, "tb-sm-content", |html| {
                            StartmenuContent::new()
                                .group(|group| group
                                    .item("Windows Update", "https://win98icons.alexmeub.com/icons/png/windows_update_small-2.png", "")
                                )
                                .group(|group| group
                                    .item("Programs", "../res/icons/programs-9.png", "")
                                    .item("Favorites", "https://win98icons.alexmeub.com/icons/png/directory_favorites_small-4.png", "")
                                    .item("Documents", "https://win98icons.alexmeub.com/icons/png/directory_open_file_mydocs_cool-3.png", "")
                                    .item("Settings", "https://win98icons.alexmeub.com/icons/png/settings_gear-4.png", "")
                                    .item("Find", "https://win98icons.alexmeub.com/icons/png/search_file_2-4.png", "")
                                    .item("Help", "https://win98icons.alexmeub.com/icons/png/help_book_small-2.png", "")
                                    .item("Run", "https://win98icons.alexmeub.com/icons/png/application_hourglass_small-2.png", "")
                                )
                                .group(|group| group
                                    .item("Log Off Kurtson...", "https://win98icons.alexmeub.com/icons/png/key_win-2.png", "")
                                    .item("Shut Down", "https://win98icons.alexmeub.com/icons/png/shut_down_normal-2.png", "")
                                )
                                .build(html, css, self);
                        });
                    });
                });
            });
            emit_div(html, "tb-item tb-quick-launch", |html| {
                fn emit_quick_launch_item(
                    html: &mut String,
                    css: &mut String,
                    config: &Config,
                    app_id: u64,
                ) {
                    assert!(Config::QUICK_LAUNCH_SUPPORTED_APPS.contains(&app_id));
                    let icon = match app_id {
                        Config::FILE_EXPLORER_ID => {
                            "https://win98icons.alexmeub.com/icons/png/computer_explorer-5.png"
                        }
                        Config::INTERNET_EXPLORER_ID => {
                            "https://win98icons.alexmeub.com/icons/png/msie1-2.png"
                        }
                        _ => panic!(),
                    };
                    emit_div(html, "tb-ql-item-container", |html| {
                        emit_div(
                            html,
                            &format!("tb-ql-item tb-ql-item-{} border-style-light-2", app_id),
                            |html| {
                                html.push_str(&format!(r##"<img src="{}" />"##, icon));
                            },
                        );
                        emit_div(
                            html,
                            &format!(
                                "tb-ql-item tb-ql-item-close tb-ql-item-close-{} border-style-light-2",
                                app_id
                            ),
                            |html| {
                                html.push_str(&format!(r##"<img src="{}" />"##, icon));
                            },
                        );
                    });
                    config.emit_action(
                        css,
                        &Action::Open(app_id),
                        &format!(".tb-ql-item-{}:active", app_id),
                    );
                    config.emit_action(
                        css,
                        &Action::Close(app_id),
                        &format!(".tb-ql-item-close-{}:active", app_id),
                    );
                }
                emit_quick_launch_item(html, css, self, Self::FILE_EXPLORER_ID);
                emit_quick_launch_item(html, css, self, Self::INTERNET_EXPLORER_ID);
            });
            emit_div(html, "tb-item tb-apps", |html| {
                let windows = self.state.borrow().windows.clone();
                for (id, name, icon) in windows {
                    emit_div(html, &format!("tb-app tb-app-{}", id), |html| {
                        emit_div(
                            html,
                            &format!("tb-app-inner border-style-asymmetric-1"),
                            |html| {
                                html.push_str(&format!(r##"<img src="{}" />"##, icon));
                                emit_p(html, "", &name);
                            },
                        );
                    });
                }
            });
            emit_div(html, "tb-item tb-right", |html| {
                emit_div(html, "tb-right-button border-style-light-2", |html| {
                    html.push_str(r##"<img src="../res/icons/taskbar-right-combined-9.png" />"##);
                    emit_div(html, "date-time-opener", |html| {
                        emit_div(html, "time-hour", |html| {
                            // for i in 0..24 {
                            //     emit_p(html, &format!("time-hour-{}", i), &format!("{:2}", i));
                            // }
                        });
                        emit_p(html, "", ":");
                        emit_div(html, "time-minute", |html| {
                            // for i in 0..60 {
                            //     emit_p(html, &format!("time-minute-{}", i), &format!("{:2}", i));
                            // }
                        });
                    });
                    // emit_p(html, "date-time-opener", "10:12");
                });
                self.emit_action(
                    css,
                    &Action::Open(Self::DATE_TIME_PROPERTIES_ID),
                    ".date-time-opener:active",
                );
            });
            //
        });
    }
    fn emit_fe_window(&self, html: &mut String, css: &mut String) {
        self.emit_window(
            html,
            css,
            Self::FILE_EXPLORER_ID,
            "File Explorer",
            "https://win98icons.alexmeub.com/icons/png/computer_explorer-5.png",
            None,
            |html, css| {
                emit_div(html, "window-header", |html| {
                    MenubarBuilder::new()
                        .item("File", |item| {
                            item
                                .group(|group| {
                                    group.sub("Exit", |i| i.action(Action::Close(Self::FILE_EXPLORER_ID)))
                                })
                        })
                        .item("Help", |item| {
                            item.group(|group| group.sub_disabled("Help Topics"))
                                .group(|group| group.sub("About", |i| i.dummy()))
                        })
                        .build(html, css, self);
                        emit_div(html, "fe-address-bar border-style-light-1", |html| {
                            emit_p(html, "", "Address");
                            emit_div(html, "fe-addrb-anchor border-style-dark-1", |html| {
                                emit_div(html, "fe-addrb-blocker", |_| ());
                                self.fs.visit_all_folders(|folder, path| {
                                    let icon = self.icon_of_folder(folder);
                                    emit_p(html, &format!("fe-addrb-path fe-addrb-{}", path.hashed()), &self.path_as_windows98y(path));
                                    css.push_str(&format!(r##".fe-addrb-{}::before {{ background: url("{}") top left / cover; }}"##, path.hashed(), icon));
                                });
                            });
                        });
                });
                emit_div(html, "window-main", |html| {
                    emit_div(html, "fe-main", |html| {
                        emit_div(html, "fe-sideview border-style-light-1", |html| {
                            html.push_str(
                                r##"<div class="fe-sideview-header">
                                                <p>All Folders</p>
                                            </div>"##,
                            );
                            emit_div(html, "fe-sideview-view", |html| {
                                fn emit_folder(html: &mut String, css: &mut String, folder: &Folder, path: PathBuf, config: &Config) {
                                    let folder_hash = path.hashed();
                                    let mut sub_folder_count = 0;
                                    emit_div(html, &format!("fe-svv-child fe-svv-child-{}", folder_hash), |html| {
                                        for (name, entry) in &folder.children {
                                            if let FsEntry::Folder(sub_folder) = &entry {
                                                sub_folder_count += 1;
                                                let expandable = (sub_folder.children.iter().filter(|x| x.1.is_folder()).collect::<Vec<_>>().len() > 0).then_some("fe-svv-expandable").unwrap_or("");
                                                emit_div(html, "fe-svv-item", |html| {
                                                    let mut sub_path = path.clone();
                                                    sub_path.push(name);
                                                    html.push_str(&format!(
                                                        r##"
                                                        <div class="fe-svvi-group {1}">
                                                            <div class="fe-svvi-expander fe-svvi-expander-open"></div>
                                                            <div class="fe-svvi-expander fe-svvi-expander-close"></div>
                                                            <div class="fe-svvi-name fe-svvi-name-{2}"><p>{0}</p> <div class="desktop-item-double-clicker"></div></div>
                                                            <div class="desktop-item-animator">
                                                                <div class="desktop-item-animator-helper"></div>
                                                            </div>
                                                        </div>"##,
                                                        name, expandable, sub_path.hashed()
                                                    ));
                                                    let condition = format!(".fe-svvi-name-{0} + .desktop-item-animator .desktop-item-animator-helper:hover", sub_path.hashed());
                                                    config.emit_action(css, &Action::OpenFileExplorer(sub_path.hashed()), &condition);
                                                    emit_folder(html, css, sub_folder, sub_path, config);
                                                });
                                            }
                                        }
                                    });
                                    // css.push_str(&format!(r##"
                                    // .fe-svv-child-{}::before {{
                                    //     height: {}px;
                                    // }}
                                    // "##, folder_hash, 14 * sub_folder_count - 3));
                                    dbg!(folder.children.len(), path);
                                }
                                emit_folder(html, css, &self.fs.root.as_folder().unwrap(), PathBuf::from("/"), self);
                            });
                            emit_div(html, "fe-sideview-resizer-parent", |html| {
                                emit_div(html, "fe-sideview-resizer fe-sideview-resizer-left", |_| {});
                                emit_div(html, "fe-sideview-resizer fe-sideview-resizer-right", |_| {});
                            });
                        });
                        emit_div(html, "fe-view-anchor", |html| {
                            fn emit_folder(config: &Config, html: &mut String, css: &mut String, folder: &Folder, path: PathBuf) {
                                let folder_hash = path.hashed();
                                emit_div(html, &format!("fe-view border-style-dark-1 fe-view-{}", folder_hash), |html| {
                                    html.push_str(&format!(r##"
                                        <h1>{}</h1>
                                    "##, path.file_name().unwrap_or(OsStr::new("root")).to_str().unwrap()));
                                    emit_div(html, "fe-view-content", |html| {
                                        config.emit_file_view_content(html, css, &path, true);
                                    });
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
            },
        );
    }
    fn emit_np_window(&self, html: &mut String, css: &mut String) {
        let fonts = [
            // NOTE: sans-serif twice because css transitions...
            ("Sans Serif", "sans-serif, sans-serif"),
            ("Courier New", "'Courier New', Courier, monospace"),
            (
                "Franklin Gothic Medium",
                "'Franklin Gothic Medium', 'Arial Narrow', Arial, sans-serif",
            ),
            (
                "Lucida Sans",
                "'Lucida Sans', 'Lucida Sans Regular', 'Lucida Grande', 'Lucida Sans Unicode', Geneva, Verdana, sans-serif",
            ),
            ("Times New Roman", "'Times New Roman', Times, serif"),
            (
                "Trebuchet MS",
                "'Trebuchet MS', 'Lucida Sans Unicode', 'Lucida Grande', 'Lucida Sans', Arial, sans-serif",
            ),
            ("Arial", "Arial, Helvetica, sans-serif"),
            (
                "Impact",
                "Impact, Haettenschweiler, 'Arial Narrow Bold', sans-serif",
            ),
            ("Verdana", "Verdana, Geneva, Tahoma, sans-serif"),
            ("Cursive", "cursive"),
            ("Fantasy", "fantasy"),
            (
                "Segoe UI",
                "'Segoe UI', Tahoma, Geneva, Verdana, sans-serif",
            ),
            ("Wingdings", "wingdings"),
        ];
        let default_font = "Sans Serif";
        let sizes = [
            "8", "9", "10", "11", "12", "14", "16", "18", "20", "22", "24", "26", "28", "36", "72",
        ];
        let default_size = "12";
        let styles = ["Regular", "Italic", "Bold", "Bold Italic"];
        let default_style = "Regular";
        {
            css.push_str(r##"
.npf-main:has(.npf-style .vertical-select-item:nth-of-type(1):active) .npf-sample p,
.window-kind-notepad:has(+ .window .npf-style .vertical-select-item:nth-of-type(1):active) .np-view {
    transition: font-family 10s linear 2147483640s, font-weight 0s linear, font-style 0s linear, font-size 10s linear 2147483640s;
    font-weight: 100;
    font-style: normal;
}
.npf-main:has(.npf-style .vertical-select-item:nth-of-type(2):active) .npf-sample p,
.window-kind-notepad:has(+ .window .npf-style .vertical-select-item:nth-of-type(2):active) .np-view {
    transition: font-family 10s linear 2147483640s, font-weight 0s linear, font-style 0s linear, font-size 10s linear 2147483640s;
    font-weight: 100;
    font-style: italic;
}
.npf-main:has(.npf-style .vertical-select-item:nth-of-type(3):active) .npf-sample p,
.window-kind-notepad:has(+ .window .npf-style .vertical-select-item:nth-of-type(3):active) .np-view {
    transition: font-family 10s linear 2147483640s, font-weight 0s linear, font-style 0s linear, font-size 10s linear 2147483640s;
    font-weight: bold;
    font-style: normal;
}
.npf-main:has(.npf-style .vertical-select-item:nth-of-type(4):active) .npf-sample p,
.window-kind-notepad:has(+ .window .npf-style .vertical-select-item:nth-of-type(4):active) .np-view {
    transition: font-family 10s linear 2147483640s, font-weight 0s linear, font-style 0s linear, font-size 10s linear 2147483640s;
    font-weight: bold;
    font-style: italic;
}
            "##);
        }
        for (i, &size) in sizes.iter().enumerate() {
            css.push_str(&format!(r##"
                .npf-main:has(.npf-size .vertical-select-item:nth-of-type({0}):active) .npf-sample p,
                .window-kind-notepad:has(+ .window .npf-size .vertical-select-item:nth-of-type({0}):active) .np-view {{
                    transition: font-family 10s linear 2147483640s, font-weight 10s linear 2147483640s, font-style 10s linear 2147483640s, font-size 0s linear;
                    font-size: {1}px;
                }}
            "##, i + 1, size));
        }
        for (i, (_, css_font)) in fonts.iter().enumerate() {
            css.push_str(&format!(r##"
                .npf-main:has(.npf-font .vertical-select-item:nth-of-type({0}):active) .npf-sample p,
                .window-kind-notepad:has(+ .window .npf-font .vertical-select-item:nth-of-type({0}):active) .np-view {{
                    transition: font-family 0s linear, font-weight 10s linear 2147483640s, font-style 10s linear 2147483640s, font-size 10s linear 2147483640s;
                    font-family: {1};
                }}
            "##, i + 1, css_font));
        }
        {
            css.push_str(&format!(
                r##"
                .main:has(.onload:hover) .npf-sample p,
                .main:has(.onload:hover) .np-view {{
                    transition: all 0s linear;
                    font-weight: 100;
                    font-style: normal;
                    font-family: {};
                    font-size: {}px;
                }}
            "##,
                // TODO: this is so cursed
                HashMap::from(fonts)[default_font],
                default_size
            ));
        }

        self.fs.visit_all_files(|file, path| {
            if file.kind != FileKind::Text {
                return;
            }
            let id = path.hashed();
            let font_id = id + 1;
            let filename = path.file_name().unwrap().to_str().unwrap();
            self.emit_window(
                html,
                css,
                id,
                &format!("{} - Notepad", filename),
                "https://win98icons.alexmeub.com/icons/png/notepad-5.png",
                Some("window-kind-notepad"),
                |html, css| {
                    emit_div(html, "window-header", |html| {
                        let word_wrap_toggle_id = 8336941761795208;
                        css.push_str(&format!(
                            r##"
                            .window-{}:has(.mb-submenu-item-{}[open]) .np-view {{
                                white-space: pre-wrap;
                                word-break: break-all;
                            }}
                        "##,
                            id, word_wrap_toggle_id
                        ));
                        MenubarBuilder::new()
                            .short(true)
                            .item("File", |item| {
                                item.group(|group| {
                                    group
                                        .sub_disabled("New")
                                        .sub_disabled("Open...")
                                        .sub_disabled("Save")
                                        .sub_disabled("Save As...")
                                })
                                .group(|group| {
                                    group.sub_disabled("Page Setup...").sub_disabled("Print")
                                })
                                .group(|group| group.sub("Exit", |i| i.action(Action::Close(id))))
                            })
                            .item("Edit", |item| {
                                item.group(|group| group.sub_disabled("Undo"))
                                    .group(|group| {
                                        group
                                            .sub_disabled("Cut")
                                            .sub_disabled("Copy")
                                            .sub_disabled("Paste")
                                            .sub_disabled("Delete")
                                    })
                                    .group(|group| {
                                        group.sub_disabled("Select All").sub_disabled("Time/Date")
                                    })
                                    .group(|group| {
                                        group
                                            .sub("Word Wrap", |i| {
                                                i.html_toggle().id(word_wrap_toggle_id)
                                            })
                                            .sub("Set Font", |i| i.action(Action::Open(font_id)))
                                    })
                            })
                            .item("Search", |item| {
                                item.group(|group| {
                                    group.sub_disabled("Find...").sub_disabled("Find Next")
                                })
                            })
                            .item("Help", |item| {
                                item.group(|group| group.sub_disabled("Help Topics"))
                                    .group(|group| group.sub("About Notepad", |i| i.dummy()))
                            })
                            .build(html, css, self);
                    });
                    //
                    emit_div(html, "window-main window-main-nopadtop", |html| {
                        emit_div(html, "np-main", |html| {
                            // emit_div(html, "fe-view-anchor", |html| {
                            //     fn emit_folder(config: &Config, html: &mut String, css: &mut String, folder: &Folder, path: PathBuf) {
                            //         let folder_hash = path.hashed();
                            emit_div(html, &format!("np-view border-style-dark-1"), |html| {
                                let path = Path::new("./res").join(&file.link);
                                let content = fs::read_to_string(&path).expect(&format!(
                                    "resource {} does not exist at {}",
                                    &file.link,
                                    path.to_str().unwrap()
                                ));
                                html.push_str(&format!(
                                    r##"
                                        <p>{}</p>
                                    "##,
                                    content.replace("<", "&lt;").replace(">", "&gt;")
                                ));
                                // config.emit_file_view_content(html, css, &path, true);
                            });
                            // for (name, entry) in &folder.children {
                            //     if let FsEntry::Folder(sub_folder) = &entry {
                            //         let mut sub_path = path.clone();
                            //         sub_path.push(name);
                            //         emit_folder(config, html, css, sub_folder, sub_path);
                            //     }
                            // }
                            //         }
                            //         emit_folder(self, html, css, &self.fs.root.as_folder().unwrap(), PathBuf::from("/"));
                            //     });
                            // });
                        })
                    });
                },
            );
            self.emit_window(
                html,
                css,
                font_id,
                "Font",
                "https://win98icons.alexmeub.com/icons/png/font_tt-0.png",
                None,
                |html, css| {
                    emit_div(html, "window-main", |html| {
                        emit_div(html, "npf-main", |html| {
                            emit_div(html, "npf-upper", |html| {
                                emit_div(html, "npf-font npf-upper-sub", |html| {
                                    emit_p(html, "", "Font:");
                                    emit_vertical_select(
                                        self,
                                        html,
                                        css,
                                        &fonts.iter().map(|x| x.0).collect::<Vec<_>>(),
                                        default_font,
                                    );
                                });
                                emit_div(html, "npf-style npf-upper-sub", |html| {
                                    emit_p(html, "", "Font style:");
                                    // NOTE: must change in notepad-font.css as well
                                    emit_vertical_select(self, html, css, &styles, default_style);
                                });
                                emit_div(html, "npf-size npf-upper-sub", |html| {
                                    emit_p(html, "", "Size:");
                                    // NOTE: must change in notepad-font.css as well
                                    emit_vertical_select(self, html, css, &sizes, default_size);
                                });
                                emit_div(html, "npf-confirm", |html| {
                                    emit_p(html, "npf-pad-p", "-");
                                    emit_p(
                                        html,
                                        "npf-ok npf-button border-style-asymmetric-1",
                                        "OK",
                                    );
                                    self.emit_action(
                                        css,
                                        &Action::Close(font_id),
                                        ".npf-button:active",
                                    );
                                    // emit_p(html, "npf-cancel npf-button border-style-asymmetric-1", "Cancel");
                                });
                            });
                            emit_div(html, "npf-lower", |html| {
                                emit_div(html, "npf-sample border-style-light-1", |html| {
                                    emit_div(
                                        html,
                                        "npf-sample-inner border-style-dark-1",
                                        |html| {
                                            emit_p(html, "", "AaBbYyZz");
                                        },
                                    );
                                });
                                // emit_div(html, "npf-script", |html| {});
                            });
                        });
                    });
                },
            );
        });
    }
    fn emit_qv_window(&self, html: &mut String, css: &mut String) {
        self.fs.visit_all_files(|file, path| {
            if file.kind != FileKind::Image {
                return;
            }
            let id = path.hashed();
            let filename = path.file_name().unwrap().to_str().unwrap();
            self.emit_window(
                html,
                css,
                id,
                &format!("{} - Quick View", filename),
                "https://win98icons.alexmeub.com/icons/png/magnifying_glass-0.png",
                None,
                |html, css| {
                    emit_div(html, "window-header", |html| {
                        // let word_wrap_toggle_id = 8336941761795208;
                        // css.push_str(&format!(
                        //     r##"
                        //     .window-{}:has(.mb-submenu-item-{}[open]) .np-view {{
                        //         white-space: pre-wrap;
                        //         word-break: break-all;
                        //     }}
                        // "##,
                        //     id, word_wrap_toggle_id
                        // ));
                        MenubarBuilder::new()
                            .short(true)
                            .item("File", |item| {
                                item.group(|group| group.sub_disabled("Open File for Editing"))
                                    .group(|group| {
                                        group.sub("Exit", |i| i.action(Action::Close(id)))
                                    })
                            })
                            .item("View", |item| {
                                item.group(|group| {
                                    group
                                        .sub_disabled("Toolbar")
                                        .sub_disabled("Status Bar")
                                        .sub_disabled("Page View")
                                        .sub_disabled("Replace Window")
                                })
                                .group(|group| {
                                    group.sub_disabled("Landscape").sub_disabled("Rotate")
                                })
                                .group(|group| group.sub_disabled("Font..."))
                            })
                            .item("Help", |item| {
                                item.group(|group| group.sub_disabled("Help Topics"))
                                    .group(|group| group.sub("About", |i| i.dummy()))
                            })
                            .build(html, css, self);
                    });
                    //
                    emit_div(html, "window-main window-main-nopadtop", |html| {
                        emit_div(html, "qv-main", |html| {
                            // emit_div(html, "fe-view-anchor", |html| {
                            //     fn emit_folder(config: &Config, html: &mut String, css: &mut String, folder: &Folder, path: PathBuf) {
                            //         let folder_hash = path.hashed();
                            emit_div(html, &format!("qv-view border-style-dark-1"), |html| {
                                let path = Path::new("./res").join(&file.link);
                                let _ = path
                                    .metadata()
                                    .expect(&format!("resource {} does not exist", &file.link));
                                html.push_str(&format!(
                                    r##"
                                        <img src="{}" />
                                    "##,
                                    // TODO: fix!
                                    &format!("../{}", path.to_str().unwrap()),
                                ));
                            });
                        })
                    });
                },
            );
        });
    }
    fn emit_ie_window(&self, html: &mut String, css: &mut String) {
        let home_domain = "foo.bar";
        let sites = read_sites();
        // let home_page = &sites[home_domain].pages[""];
        let mut history_items = Vec::new();
        for (_, site) in &sites {
            for (path, _page) in &site.pages {
                dbg!(format!("{}-{}", site.domain, path),);
                history_items.push(HistoryItem {
                    id: dbg!(format!("{}-{}", site.domain, path).hashed()),
                    rules: vec![
                        format!(
                            r##".ie-site-{} {{
                                transition: 0s;
                                z-index: 2147483640;
                            }}"##,
                            site.domain.hashed()
                        ),
                        format!(
                            r##".ie-page-{} {{
                                transition: 0s;
                                z-index: 2147483640;
                            }}"##,
                            path.hashed()
                        ),
                    ],
                });
            }
        }
        // for i in 0..25 {
        //     history_items.push(HistoryItem {
        //         id: i + 1,
        //         rule: format!(""),
        //     });
        // }
        let default_id = format!("{}-{}", home_domain, "").hashed();
        let history = History::new(history_items, default_id);
        history.emit_stack(html, css, self);
        self.emit_window(
            html,
            css,
            Self::INTERNET_EXPLORER_ID,
            "Internet Explorer",
            "https://win98icons.alexmeub.com/icons/png/html-5.png",
            None,
            |html, css| {
                // TODO: window resizing and shit
                css.push_str(&format!(
                    r##".window.window-{} {{
                        width: 600px;
                        height: 600px;
                    }}"##,
                    Self::INTERNET_EXPLORER_ID
                ));
                emit_div(html, "window-header", |html| {
                    MenubarBuilder::new()
                        .item("File", |item| {
                            item.group(|group| {
                                group.sub("Exit", |i| {
                                    i.action(Action::Close(Self::INTERNET_EXPLORER_ID))
                                })
                            })
                        })
                        // .item("View", |item| {
                        //     item.group(|group| {
                        //         group
                        //             .sub_disabled("Toolbar")
                        //             .sub_disabled("Status Bar")
                        //             .sub_disabled("Page View")
                        //             .sub_disabled("Replace Window")
                        //     })
                        //     .group(|group| {
                        //         group.sub_disabled("Landscape").sub_disabled("Rotate")
                        //     })
                        //     .group(|group| group.sub_disabled("Font..."))
                        // })
                        .item("Help", |item| {
                            item.group(|group| group.sub_disabled("Help Topics"))
                                .group(|group| group.sub("About", |i| i.dummy()))
                        })
                        .build(html, css, self);
                    ToolbarBuilder::new()
                        .group(|group| {
                            group
                                // .item("Back", "../res/icons/back-9.png")
                                // .item("Forward", "../res/icons/forward-9.png")
                                .item_html(|html| {
                                    emit_div(html, "history-toolbar-container", |html| {
                                        emit_div(html, "toolbar-item history-back", |html| {
                                            html.push_str(&format!(
                                                r##"
                                                <img src="{}" />
                                                <p>{}</p>
                                            "##,
                                                "../res/icons/back-9.png", "Back"
                                            ));
                                        });
                                        emit_div(html, "toolbar-item toolbar-single-click", |_| ());
                                        emit_div(html, "toolbar-item toolbar-disabled", |html| {
                                            html.push_str(&format!(
                                                r##"
                                                <img src="{}" />
                                                <p>{}</p>
                                            "##,
                                                "../res/icons/back-no-9.png", "Back"
                                            ));
                                        });
                                    });
                                })
                                .item_html(|html| {
                                    emit_div(html, "history-toolbar-container", |html| {
                                        emit_div(html, "toolbar-item history-forward", |html| {
                                            html.push_str(&format!(
                                                r##"
                                            <img src="{}" />
                                            <p>{}</p>
                                        "##,
                                                "../res/icons/forward-9.png", "Forward"
                                            ));
                                        });
                                        emit_div(html, "toolbar-item toolbar-single-click", |_| ());
                                        emit_div(html, "toolbar-item toolbar-disabled", |html| {
                                            html.push_str(&format!(
                                                r##"
                                            <img src="{}" />
                                            <p>{}</p>
                                        "##,
                                                "../res/icons/forward-no-9.png", "Forward"
                                            ));
                                        });
                                    });
                                })
                                .item("Stop", "../res/icons/stop-9.png", "ie-tb-stop")
                                .item("Refresh", "../res/icons/refresh-9.png", "ie-tb-refresh")
                                .item(
                                    "Home",
                                    "../res/icons/home-9.png",
                                    &format!("history-trigger history-trigger-{}", default_id),
                                )
                        })
                        .group(|group| {
                            group
                                .item("Search", "../res/icons/search-9.png", "ie-tb-search")
                                .item(
                                    "Favorites",
                                    "../res/icons/favorites-9.png",
                                    "ie-tb-favorites",
                                )
                                .item("History", "../res/icons/history-9.png", "ie-tb-history")
                        })
                        .group(|group| {
                            group
                                .item("Mail", "../res/icons/mail-9.png", "ie-tb-mail")
                                .item("Print", "../res/icons/print-9.png", "ie-tb-print")
                        })
                        .build(html, css, self);
                    emit_div(html, "ie-address-bar border-style-light-1", |html| {
                        html.push_str(&format!(
                            r##"
                            <p>Address</p>
                            <p class="border-style-dark-1">My Computer</p>
                        "##
                        ));
                    });
                });
                //
                emit_div(html, "window-main", |html| {
                    emit_div(html, "ie-main", |html| {
                        emit_div(html, "ie-view border-style-dark-1", |html| {
                            for (domain, site) in &sites {
                                css.push_str(&site.global_css);
                                emit_div(
                                    html,
                                    &format!("ie-site ie-site-{}", domain.hashed()),
                                    |html| {
                                        for (page_path, page) in &site.pages {
                                            emit_div(
                                                html,
                                                &format!("ie-page ie-page-{}", page_path.hashed()),
                                                |html| {
                                                    html.push_str(&page.html);
                                                },
                                            );
                                        }
                                        // emit_div(html, "ie-page-cover", |_| ());
                                    },
                                );
                            }
                        });
                    });
                });
            },
        );
    }
    fn emit_dt_window(&self, html: &mut String, css: &mut String) {
        assert!(Self::DATE_TIME_PROPERTIES_ID == 420); // change in css if changed!
        self.emit_window(
            html,
            css,
            Self::DATE_TIME_PROPERTIES_ID,
            "Date/Time Properties",
            "https://win98icons.alexmeub.com/icons/png/time_and_date-4.png",
            None,
            |html, css| {
                emit_div(html, "window-main", |html| {
                    emit_div(html, "dt-main", |html| {
                        emit_p(html, "dt-header border-style-asymmetric-1", "Date & Time");
                        emit_div(html, "dt-view border-style-asymmetric-1", |html| {
                            emit_div(html, "dt-upper", |html| {
                                emit_div(html, "dt-date border-style-light-1", |html| {
                                    emit_div(html, "dt-date-upper", |html| {
                                        emit_p(html, "dt-month border-style-dark-1", "November");
                                        emit_p(html, "dt-year border-style-dark-1", "2001");
                                    });
                                    emit_div(html, "dt-calendar border-style-dark-1", |html| {
                                        emit_div(html, "dt-calendar-header", |html| {
                                            emit_p(html, "", "M");
                                            emit_p(html, "", "T");
                                            emit_p(html, "", "O");
                                            emit_p(html, "", "T");
                                            emit_p(html, "", "F");
                                            emit_p(html, "", "L");
                                            emit_p(html, "", "S");
                                        });
                                        emit_div(html, "dt-calendar-body", |html| {
                                            let mut count = -2;
                                            for row in 0..5 {
                                                emit_div(html, "dt-calendar-row", |html| {
                                                    for col in 0..7 {
                                                        if count > 0 && count <= 30 {
                                                            emit_p(html, "", &format!("{count}"));
                                                        } else {
                                                            emit_p(html, "", "");
                                                        }
                                                        count+= 1;
                                                    }
                                                });
                                            }
                                        });
                                        //
                                    });
                                });
                                emit_div(html, "dt-time border-style-light-1", |html| {
                                    emit_div(html, "dt-clock", |html| {
                                        emit_div(html, "dt-second-hand", |_| {});
                                        emit_div(html, "dt-minute-hand", |_| {});
                                        emit_div(html, "dt-hour-hand", |_| {});
                                        //
                                    });
                                    emit_div(html, "dt-time-display border-style-dark-1", |html| {
                                        emit_div(html, "time-hour", |_| {});
                                        emit_p(html, "", ":");
                                        emit_div(html, "time-minute", |_| {});
                                    });
                                });
                            });
                            emit_div(html, "dt-timezone-panel border-style-light-1", |html| {
                                emit_p(
                                    html,
                                    "dt-timezone border-style-dark-1",
                                    "(GMT+01:00) Amsterdam, Berlin, Bern, Rome, Stockholm, Vienna",
                                );
                                emit_div(html, "dt-timezone-bottom", |html| {
                                    // emit_div(html, "checkbox border-style-dark-1", |html| {
                                    //     //
                                    // });
                                    html.push_str(r##"<details class="checkbox border-style-dark-1 dt-timezone-checkbox"><summary></summary></details>"##);
                                    emit_p(
                                        html,
                                        "dt-timezone-check",
                                        "Automatically adjust clock for daylight saving changes",
                                    );
                                });
                            });
                        });
                        emit_div(html, "dt-footer", |html| {
                            emit_p(html, "dt-ok dt-button border-style-asymmetric-1", "OK");
                            self.emit_action(
                                css,
                                &Action::Close(Self::DATE_TIME_PROPERTIES_ID),
                                ".dt-ok:active",
                            );
                        });
                    })
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
        extra_classes: Option<&str>,
        mut cb: impl FnMut(&mut String, &mut String),
    ) {
        if !self
            .state
            .borrow_mut()
            .windows
            .insert((id, name.into(), icon.into()))
        {
            panic!("the fook are you doing?");
        }
        html.push_str(&format!(
            r##"
            <div class="window window-{0} {2}">
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
            extra_classes.unwrap_or("")
        ));
        cb(html, css);
        html.push_str(
            r##"
                </div>
            </div>
        "##,
        );
        if Self::QUICK_LAUNCH_SUPPORTED_APPS.contains(&id) {
            // we need additional logic in this case!
            self.emit_action(
                css,
                &Action::Close(id),
                &format!(".window-{0} .window-exiter:active", id),
            );
        }
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
        is_in_explorer: bool,
        // mut cb: impl FnMut(&mut String, &mut String),
    ) {
        // dbg!(path);
        let folder = self.fs_entry(path).unwrap().as_folder().unwrap();
        for (i, (name, entry)) in folder.children.iter().enumerate() {
            // dbg!(name);
            let mut entry_path = path.to_owned();
            entry_path.push(name);
            let unique_hash: u64 = rand::rng().random();
            // let unique_hash =
            //     format!("{}___{}", entry_path.to_str().unwrap(), some_random_numer).hashed();
            let mut offset = entry.offset().unwrap_or_default();
            if is_in_explorer {
                let cols = 5;
                offset = (i as u32 % cols, i as u32 / cols);
            }
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
            self.emit_fe_interaction(css, entry, &entry_path, unique_hash, is_in_explorer);
        }
    }
    fn emit_fe_interaction(
        &self,
        css: &mut String,
        entry: &FsEntry,
        entry_path: &Path,
        file_unique_hash: u64,
        is_in_explorer: bool,
    ) {
        let condition = format!(
            ".desktop-item-{0} + .desktop-item-animator .desktop-item-animator-helper:hover",
            file_unique_hash
        );
        if let Some(file) = entry.as_file() {
            match file.kind {
                FileKind::App => {
                    self.emit_action(css, &Action::Open(file.link.hashed()), &condition);
                }
                FileKind::Shortcut => {
                    let new_path = Path::new(&file.link);
                    let new_entry = self.fs_entry(new_path).unwrap();
                    self.emit_fe_interaction(
                        css,
                        new_entry,
                        new_path,
                        file_unique_hash,
                        is_in_explorer,
                    );
                }
                FileKind::Text | FileKind::Image => {
                    self.emit_action(css, &Action::Open(entry_path.hashed()), &condition);
                }
                FileKind::NativeApp => {
                    self.emit_action(css, &Action::Open(file.link.parse().unwrap()), &condition);
                }
            }
        }
        if let Some(_sub_folder) = entry.as_folder() {
            if !is_in_explorer {
                self.emit_action(css, &Action::Open(Self::FILE_EXPLORER_ID), &condition);
            }
            self.emit_action(
                css,
                &Action::OpenFileExplorer(entry_path.hashed()),
                &condition,
            );
        }
    }
    fn emit_action(&self, css: &mut String, action: &Action, condition: &str) {
        match action {
            Action::Close(id) => {
                css.push_str(&format!(
                    r##"
                    .main:has({0}) .window-{1} {{
                        top: 0.002px;
                        left: -2000.002px;
                        transition: top 0s linear 0s, left 0s linear 0s !important;
                    }}
                    .main:has({0}) .tb-app-{1} {{
                        max-width: 0px;
                        transition: 0s;
                    }}
                    "##,
                    condition, id,
                ));
                if Self::QUICK_LAUNCH_SUPPORTED_APPS.contains(&id) {
                    css.push_str(&format!(
                        r##"
                        .main:has({0}) .tb-ql-item-close-{1} {{
                            transition: 0s;
                            z-index: -2;
                        }}
                        "##,
                        condition, id,
                    ));
                }
            }
            Action::Open(id) => {
                // TODO: this is Really shitty (tm). Also why the FUCK does this work (especially with touchpad taps????). Investigate!!!!
                // NOTE: we have extra ".window"s because css selector specificity...
                css.push_str(&format!(
                    r##"
                    .main:has({0}) .window-{1}.window.window {{
                        top: 30px;
                        left: 30px;
                        transition: top 0s linear 0s, left 0s linear 0s, z-index 0s linear;
                        z-index: 2147483640;
                    }}

.main:has({0}) .window:not(.window-{1}).window.window {{
    z-index: 1;
    transition: left 10s linear 2147483640s, top 10s linear 2147483640s, z-index 214748s linear;
}}
.main:has({0}) .window-{1}.window.window .window-titlebar {{
    background: linear-gradient(to right, #00007B, #3B79B8);
    transition: background 0s linear;
}}
.main:has({0}) .window:not(.window-{1}).window.window .window-titlebar {{
    background: linear-gradient(to right, rgb(126, 126, 125), rgb(187, 187, 187));
    transition: background 0s linear;
}}
.main:has({0}) .tb-app-{1} {{
    max-width: 160px;
    transition: 0s;
}}
                    "##,
                    condition, id,
                ));
                if Self::QUICK_LAUNCH_SUPPORTED_APPS.contains(&id) {
                    css.push_str(&format!(
                        r##"
                        .main:has({0}) .tb-ql-item-close-{1} {{
                            transition: 0s;
                            z-index: 2;
                        }}
                        "##,
                        condition, id,
                    ));
                }
            }
            Action::OpenFileExplorer(id) => {
                // TODO: for now we don't do this, as to not preemtively complicate design.
                // self.emit_action(css, Action::Open(Self::FILE_EXPLORER_ID), condition);
                css.push_str(&format!(
                    r##"
                    .main:has({0}) .fe-view-{1} {{
                        left: 0px;
                        transition: left 0s linear !important;
                    }}
                    .main:has({0}) .fe-view:not(.fe-view-{1}) {{
                        left: -20000.001px;
                        transition: left 0s linear !important;
                    }}
                    .main:has({0}) .fe-sideview-view .fe-svv-child:is(:has(.fe-svv-child-{1}), .fe-svv-child-{1}) {{
                        height: 100%;
                        transition: 0s;
                    }}
                    .main:has({0}) .fe-sideview-view :is(.fe-svv-expandable:has(+ .fe-svv-child-{1}), .fe-svv-expandable:has(+ .fe-svv-child .fe-svv-child-{1})) .fe-svvi-expander-open {{
                        transition: 0s;
	                    z-index: 1;
                    }}
                    .main:has({0}) .fe-addrb-{1} {{
                        transition: 0s;
	                    z-index: 2147483640;
                    }}
                    "##,
                    condition, id
                ));
            }
            Action::OpenNotepad(id) => {
                // self.emit_action(css, Action::Open(Self::NOTEPAD_ID), condition);
                todo!();
            }
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
            FsEntry::File(file) => self.icon_of_file(file),
            FsEntry::Folder(folder) => self.icon_of_folder(folder),
        }
    }
    fn icon_of_folder(&self, file: &Folder) -> String {
        // TODO: what if this resource disappears?
        "https://win98icons.alexmeub.com/icons/png/directory_closed-4.png".to_owned()
    }
    fn icon_of_file(&self, file: &File) -> String {
        match file.kind {
            FileKind::App => self.app(&file.link).icon.clone(),
            FileKind::Shortcut => self.icon_of(self.fs_entry(Path::new(&file.link)).unwrap()),
            FileKind::Text => {
                "https://win98icons.alexmeub.com/icons/png/notepad_file-2.png".to_owned()
            }
            FileKind::Image => {
                "https://win98icons.alexmeub.com/icons/png/paint_file-5.png".to_owned()
            }
            FileKind::NativeApp => {
                match file.link.parse::<u64>().expect("native app id must be u64") {
                    Self::INTERNET_EXPLORER_ID => {
                        "https://win98icons.alexmeub.com/icons/png/msie1-2.png".to_owned()
                    }
                    _ => panic!("invalid native app id"),
                }
            }
        }
    }
    fn path_as_windows98y(&self, path: &Path) -> String {
        path.to_str().unwrap().replace("/", "\\")
    }
}
fn emit_div(s: &mut String, class: &str, mut cb: impl FnMut(&mut String)) {
    s.push_str(&format!(r##"<div class="{class}">"##));
    cb(s);
    s.push_str(&format!(r##"</div>"##));
}
fn emit_p(s: &mut String, class: &str, content: &str) {
    s.push_str(&format!(r##"<p class="{class}">{content}</p>"##));
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
