use std::fs;

use crate::{
    config::{Config, File, FileKind},
    css_var_remove::css_var_remove,
};

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

pub fn build(mut config: Config) -> BuildResult {
    {
        let mut app_files = Vec::new();
        for app in &config.apps {
            if let Some(desktop_pos) = app.add_to_desktop {
                app_files.push(File {
                    kind: FileKind::App,
                    link: app.name.clone(),
                    offset: Some(desktop_pos),
                });
            }
        }
    }

    let mut css = String::new();
    let mut html = String::new();
    css.push_str(&load_css());
    emit_div(&mut html, "main", |html| {
        emit_div(html, "desktop", |html| {
            for w in config.apps.iter().filter(|x| x.add_to_desktop.is_some()) {
                let pos = w.add_to_desktop.unwrap();
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
                    w.id(),
                    w.name
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
                        .desktop:has(.desktop-item-{0} + .desktop-item-animator .desktop-item-animator-helper:hover) ~ .window-{0} {{
                            top: 30px;
                            left: 30px;
                            transition: top 0s linear 0s, left 0s linear 0s;
                        }}
                        "##,
                        w.id(),
                        w.icon,
                        pos.0 * 54,
                        pos.1 * 64,
                    ));
            }
        });
        for w in &config.apps {
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
                        w.id(),
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
                w.id(),
                w.icon,
            ));
        }
    });
    BuildResult::from_html_css(html, css)
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
