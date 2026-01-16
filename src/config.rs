use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::{Path, PathBuf};
use std::{fs, path};

use enum_as_inner::EnumAsInner;
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::config::menubar_builder::MenubarBuilder;
use crate::config::toolbar_builder::ToolbarBuilder;
use crate::config::vertical_select::emit_vertical_select;
use crate::css_var_remove::css_var_remove;

mod menubar_builder;
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
        };
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
    pub children: HashMap<String, FsEntry>,
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
            None,
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
            self.emit_window(
                html,
                css,
                id,
                "Notepad",
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
                                    emit_p(html, "npf-ok npf-button", "OK");
                                    self.emit_action(
                                        css,
                                        &Action::Close(font_id),
                                        ".npf-button:active",
                                    );
                                    // emit_p(html, "npf-cancel npf-button", "Cancel");
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
            self.emit_window(
                html,
                css,
                id,
                "Quick View",
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
                                .item("Back", "../res/icons/back-9.png")
                                .item("Forward", "../res/icons/forward-9.png")
                                .item("Stop", "../res/icons/stop-9.png")
                                .item("Refresh", "../res/icons/refresh-9.png")
                                .item("Home", "../res/icons/home-9.png")
                        })
                        .group(|group| {
                            group
                                .item("Search", "../res/icons/search-9.png")
                                .item("Favorites", "../res/icons/favorites-9.png")
                                .item("History", "../res/icons/history-9.png")
                        })
                        .group(|group| {
                            group
                                .item("Mail", "../res/icons/mail-9.png")
                                .item("Print", "../res/icons/print-9.png")
                        })
                        .build(html, css, self);
                    // emit_div(html, "ie-toolbar", |html| {
                    //     let items =
                    // });
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
                    // emit_div(html, "ie-main", |html| {
                    //     emit_div(html, "ie-sideview border-style-light-1", |html| {
                    //         html.push_str(
                    //             r##"<div class="ie-sideview-header">
                    //                             <p>All Folders</p>
                    //                         </div>"##,
                    //         );
                    //         emit_div(html, "ie-sideview-view", |html| {
                    //             fn emit_folder(html: &mut String, css: &mut String, folder: &Folder, path: PathBuf) {
                    //                 let folder_hash = path.hashed();
                    //                 let mut sub_folder_count = 0;
                    //                 emit_div(html, &format!("ie-svv-child ie-svv-child-{}", folder_hash), |html| {
                    //                     for (name, entry) in &folder.children {
                    //                         if let FsEntry::Folder(sub_folder) = &entry {
                    //                             sub_folder_count += 1;
                    //                             emit_div(html, "ie-svv-item", |html| {
                    //                                 html.push_str(&format!(
                    //                                     r##"
                    //                                     <div class="ie-svvi-group">
                    //                                         <div class="ie-svvi-expander"></div>
                    //                                         <p class="ie-svvi-name">{}</p>
                    //                                     </div>"##,
                    //                                     name
                    //                                 ));
                    //                                 let mut sub_path = path.clone();
                    //                                 sub_path.push(name);
                    //                                 emit_folder(html, css, sub_folder, sub_path);
                    //                             });
                    //                         }
                    //                     }
                    //                 });
                    //                 css.push_str(&format!(r##"
                    //                 .ie-svv-child-{}::before {{
                    //                     height: {}px;
                    //                 }}
                    //                 "##, folder_hash, 14 * sub_folder_count - 3));
                    //                 dbg!(folder.children.len(), path);
                    //             }
                    //             emit_folder(html, css, &self.fs.root.as_folder().unwrap(), PathBuf::from("/"));
                    //         });
                    //     });
                    //     emit_div(html, "ie-view-anchor", |html| {
                    //         fn emit_folder(config: &Config, html: &mut String, css: &mut String, folder: &Folder, path: PathBuf) {
                    //             let folder_hash = path.hashed();
                    //             emit_div(html, &format!("ie-view border-style-dark-1 ie-view-{}", folder_hash), |html| {
                    //                 config.emit_file_view_content(html, css, &path, true);
                    //             });
                    //             for (name, entry) in &folder.children {
                    //                 if let FsEntry::Folder(sub_folder) = &entry {
                    //                     let mut sub_path = path.clone();
                    //                     sub_path.push(name);
                    //                     emit_folder(config, html, css, sub_folder, sub_path);
                    //                 }
                    //             }
                    //         }
                    //         emit_folder(self, html, css, &self.fs.root.as_folder().unwrap(), PathBuf::from("/"));
                    //     });
                    // });
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
                    }}"##,
                    condition, id,
                ));
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
                    "##,
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
