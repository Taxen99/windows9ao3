use std::{
    fs,
    hash::{DefaultHasher, Hash, Hasher},
};

use arcthing::css_var_remove::css_var_remove;

struct Window {
    name: &'static str,
    icon: &'static str,
    add_to_desktop: Option<(u8, u8)>,
    content: &'static str,
}

impl Window {
    fn id(&self) -> u64 {
        let mut s = DefaultHasher::new();
        self.name.hash(&mut s);
        s.finish()
    }
}

struct Config {
    windows: Vec<Window>,
}

#[derive(Debug, Default)]
struct BuildResult {
    html: String,
    css: String,
    ao3_html: String,
    ao3_css: String,
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
    fn build(self) -> BuildResult {
        let mut css = String::new();
        let mut html = String::new();
        css.push_str(&Self::load_css());
        Self::emit_div(&mut html, "main", |html| {
            Self::emit_div(html, "desktop", |html| {
                for w in self.windows.iter().filter(|x| x.add_to_desktop.is_some()) {
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
            for w in &self.windows {
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
        for e in fs::read_dir("src/bin/css").unwrap() {
            let e = e.unwrap();
            assert!(e.path().extension().unwrap() == "css");
            res.push_str(std::fs::read_to_string(e.path()).unwrap().as_str());
            res.push('\n');
        }
        res
    }
}

pub fn main() {
    let config = Config {
        windows: vec![
            Window {
                name: "Internet Explorer",
                icon: "https://win98icons.alexmeub.com/icons/png/msie1-2.png",
                add_to_desktop: Some((0, 0)),
                content: "Internet Exploruvu!!!",
            },
            Window {
                name: "My Computer",
                icon: "https://win98icons.alexmeub.com/icons/png/computer_explorer-5.png",
                add_to_desktop: Some((1, 1)),
                content: "My <i>uwoepic</i> COMPUTER!!!",
            },
        ],
    };
    let res = config.build();
    let _ = fs::create_dir("output");
    // if fs::read_to_string("output/index.html").unwrap_or_default() != res.html {
    //     // avoid reloading page if html didn't change.
    fs::write("output/index.html", res.html).unwrap();
    // }
    fs::write("output/style.css", res.css).unwrap();
    fs::write("output/ao3.html", res.ao3_html).unwrap();
    fs::write("output/ao3.css", res.ao3_css).unwrap();
}
