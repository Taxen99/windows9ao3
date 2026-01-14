use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::{Path, PathBuf};
use std::{fs, path};

use enum_as_inner::EnumAsInner;
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::config::menubar_builder::MenubarBuilder;
use crate::config::vertical_select::emit_vertical_select;
use crate::css_var_remove::css_var_remove;

mod menubar_builder;
mod vertical_select;

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
    Text,
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
    const NOTEPAD_ID: u64 = 2;
    const NOTEPAD_FONT_ID: u64 = 3;
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
            });
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
                    <div class="menubar border-style-light-1">
                        <div class="menubar-item">
                            <div class="menubar-item-state"></div>
                            <p>File</p>
                            <div class="mb-submenu border-style-dark-2">
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
                    <div class="fe-address-bar border-style-light-1">
                        <p>Address</p>
                        <p class="border-style-dark-1">My Computer</p>
                    </div>
                </div>
                "##));
                //
                emit_div(html, "window-main", |html| {
                    // FIXME: this looks sussy, why two fe-mains?
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
                                        config.emit_file_view_content(html, css, &path, true);
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
    fn emit_np_window(&self, html: &mut String, css: &mut String) {
        self.emit_window(
            html,
            css,
            Self::NOTEPAD_ID,
            "Notepad",
            "https://win98icons.alexmeub.com/icons/png/notepad-5.png",
            |html, css| {
                emit_div(html, "window-header", |html| {
                    let word_wrap_toggle_id = 8336941761795208;
                    css.push_str(&format!(r##"
                        .window-{}:has(.mb-submenu-item-{}[open]) .np-view {{
	                        white-space: pre-wrap;
                            word-break: break-all;
                        }}
                    "##, Self::NOTEPAD_ID, word_wrap_toggle_id));
                    MenubarBuilder::new()
                        .short(true)
                        .item("File", |item| item
                            .group(|group| group
                                .sub_disabled("New")
                                .sub_disabled("Open...")
                                .sub_disabled("Save")
                                .sub_disabled("Save As...")
                            )
                            .group(|group| group
                                .sub_disabled("Page Setup...")
                                .sub_disabled("Print")
                            )
                            .group(|group| group
                                .sub("Exit", |i| i.action(Action::Close(Self::NOTEPAD_ID)))
                            )
                        )
                        .item("Edit", |item| item
                            .group(|group| group
                                .sub_disabled("Undo")
                            )
                            .group(|group| group
                                .sub_disabled("Cut")
                                .sub_disabled("Copy")
                                .sub_disabled("Paste")
                                .sub_disabled("Delete")
                            )
                            .group(|group| group
                                .sub_disabled("Select All")
                                .sub_disabled("Time/Date")
                            )
                            .group(|group| group
                                .sub("Word Wrap", |i| i.html_toggle().id(word_wrap_toggle_id))
                                .sub("Set Font", |i| i.action(Action::Open(Self::NOTEPAD_FONT_ID)))
                            )
                        )
                        .item("Search", |item| item
                            .group(|group| group
                                .sub_disabled("Find...")
                                .sub_disabled("Find Next")
                            )
                        )
                        .item("Help", |item| item
                            .group(|group| group
                                .sub_disabled("Help Topics")
                            )
                            .group(|group| group
                                .sub("About Notepad", |i| i.dummy())
                            )
                        )
                        .build(html, css, self);
                });
                //
                emit_div(html, "window-main window-main-nopadtop", |html| {
                    emit_div(html, "np-main", |html| {
                            // emit_div(html, "fe-view-anchor", |html| {
                            //     fn emit_folder(config: &Config, html: &mut String, css: &mut String, folder: &Folder, path: PathBuf) {
                            //         let folder_hash = path.hashed();
                                    emit_div(html, &format!("np-view border-style-dark-1"), |html| {
                                        const THINGY: &str = r###"<!DOCTYPE html>\n<html lang="en">\n\n<head>\n\t<meta charset="utf-8" />\n\t<meta http-equiv="x-ua-compatible" content="ie=edge" />\n\t<meta name="keywords" content="fanfiction, transformative works, otw, fair use, archive" />\n\t<meta name="language" content="en-US" />\n\t<meta name="subject" content="fandom" />\n\t<meta name="description" content="An Archive of Our Own, a project of the Organization for Transformative Works" />\n\t<meta name="distribution" content="GLOBAL" />\n\t<meta name="classification" content="transformative works" />\n\t<meta name="author" content="Organization for Transformative Works" />\n\t<meta name="robots" content="noindex" />\n\t<meta name="googlebot" content="noindex" />\n\t<meta name="viewport" content="width=device-width, initial-scale=1.0" />\n\t<meta name="chrome" content="nointentdetection" />\n\t<meta name="format-detection" content="telephone=no" />\n\t<title>test testson - Kurt Kurtson (taxen99) - Original Work [Archive of Our Own]</title>\n\n\t<link rel="stylesheet" type="text/css" media="screen" href="https://archiveofourown.org//stylesheets/skins/skin_1_default/1_site_screen_.css" />\n\t<link rel="stylesheet" type="text/css" media="only screen and (max-width: 62em), handheld"\n\t\thref="https://archiveofourown.org/stylesheets/skins/skin_1_default/4_site_midsize.handheld_.css" />\n\t<link rel="stylesheet" type="text/css" media="only screen and (max-width: 42em), handheld"\n\t\thref="https://archiveofourown.org/stylesheets/skins/skin_1_default/5_site_narrow.handheld_.css" />\n\t<link rel="stylesheet" type="text/css" media="speech" href="https://archiveofourown.org/stylesheets/skins/skin_1_default/6_site_speech_.css" />\n\t<link rel="stylesheet" type="text/css" media="print" href="https://archiveofourown.org/stylesheets/skins/skin_1_default/7_site_print_.css" />\n\t\x3C!--[if IE 8]><link rel="stylesheet" type="text/css" media="screen" href="/stylesheets/skins/skin_1_default/8_site_screen_IE8_or_lower.css" /><![endif]-->\n\t\x3C!--[if IE 5]><link rel="stylesheet" type="text/css" media="screen" href="/stylesheets/skins/skin_1_default/9_site_screen_IE5.css" /><![endif]-->\n\t\x3C!--[if IE 6]><link rel="stylesheet" type="text/css" media="screen" href="/stylesheets/skins/skin_1_default/10_site_screen_IE6.css" /><![endif]-->\n\t\x3C!--[if IE 7]><link rel="stylesheet" type="text/css" media="screen" href="/stylesheets/skins/skin_1_default/11_site_screen_IE7.css" /><![endif]-->\n\n\n\t\x3C!--sandbox for developers\t-->\n\t<link rel="stylesheet" href="https://archiveofourown.org/stylesheets/sandbox.css" />\n\n\n\n\t\x3Cscript src="https://archiveofourown.org/javascripts/livevalidation_standalone.js">\x3C/script>\n\n\t<meta name="csrf-param" content="authenticity_token" />\n\t<meta name="csrf-token"\n\t\tcontent="-PDRY6n50tSUYv72wKd9N3fkT2IjDC3nj9Ooa884vN7P8A4TFMGqiHM5A5-rO5_dYbV2RKZ23YekbcHtNQpucw" />\n\n\n</head>\n\n<body class="logged-in">\n\t<div id="outer" class="wrapper">\n\t\t<ul id="skiplinks">\n\t\t\t<li><a href="#main">Main Content</a></li>\n\t\t</ul>\n\t\t<noscript>\n\t\t\t<p id="javascript-warning">While we&#39;ve done our best to make the core functionality of this site\n\t\t\t\taccessible without JavaScript, it will work better with it enabled. Please consider turning it on!</p>\n\t\t</noscript>"###;

                                        html.push_str(&format!(r##"
                                            <p>{}</p>
                                        "##, &THINGY.to_owned().replace("\\n", "\n").replace("\\t", "\t").replace("<", "&lt;").replace(">", "&gt;")));
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
        ];
        let default_font = "Sans Serif";
        let sizes = [
            "8", "9", "10", "11", "12", "14", "16", "18", "20", "22", "24", "26", "28", "36", "72",
        ];
        let default_size = "12";
        let styles = ["Regular", "Italic", "Bold", "Bold Italic"];
        let default_style = "Regular";
        for (i, &size) in sizes.iter().enumerate() {
            css.push_str(&format!(r##"
                .npf-main:has(.npf-size .vertical-select-item:nth-of-type({}):active) .npf-sample p {{
                    transition: font-family 10s linear 2147483640s, font-weight 10s linear 2147483640s, font-style 10s linear 2147483640s, font-size 0s linear;
                    font-size: {}px;
                }}
            "##, i + 1, size));
        }
        for (i, (_, css_font)) in fonts.iter().enumerate() {
            css.push_str(&format!(r##"
                .npf-main:has(.npf-font .vertical-select-item:nth-of-type({}):active) .npf-sample p {{
                    transition: font-family 0s linear, font-weight 10s linear 2147483640s, font-style 10s linear 2147483640s, font-size 10s linear 2147483640s;
                    font-family: {};
                }}
            "##, i + 1, css_font));
        }
        {
            css.push_str(&format!(
                r##"
                .main:has(.onload:hover) .npf-sample p {{
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

        self.emit_window(
            html,
            css,
            Self::NOTEPAD_FONT_ID,
            "Font",
            "https://win98icons.alexmeub.com/icons/png/font_tt-0.png",
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
                                emit_p(html, "npf-ok npf-button", "OK");
                                self.emit_action(
                                    css,
                                    &Action::Close(Self::NOTEPAD_FONT_ID),
                                    ".npf-button:active",
                                );
                                // emit_p(html, "npf-cancel npf-button", "Cancel");
                            });
                        });
                        emit_div(html, "npf-lower", |html| {
                            emit_div(html, "npf-sample border-style-light-1", |html| {
                                emit_div(html, "npf-sample-inner border-style-dark-1", |html| {
                                    emit_p(html, "", "AaBbYyZz");
                                });
                            });
                            // emit_div(html, "npf-script", |html| {});
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
        is_in_explorer: bool,
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
                FileKind::Text => {
                    self.emit_action(css, &Action::Open(Self::NOTEPAD_ID), &condition);
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
                    }}"##,
                    condition, id,
                ));
            }
            Action::Open(id) => {
                css.push_str(&format!(
                    r##"
                    .main:has({0}) .window-{1} {{
                        top: 30px;
                        left: 30px;
                        transition: top 0s linear 0s, left 0s linear 0s !important;
                    }}"##,
                    condition, id,
                ));
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
            FsEntry::File(file) => match file.kind {
                FileKind::App => self.app(&file.link).icon.clone(),
                FileKind::Shortcut => self.icon_of(self.fs_entry(Path::new(&file.link)).unwrap()),
                FileKind::Text => {
                    "https://win98icons.alexmeub.com/icons/png/notepad_file-2.png".to_owned()
                }
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
