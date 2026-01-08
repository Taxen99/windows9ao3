use std::fs;

use arcthing::config::Config;

pub fn main() {
    panic!();
    // let config: Config = toml::from_str(&fs::read_to_string("config.toml").unwrap()).unwrap();
    let config: Config = ron::from_str(&fs::read_to_string("config.ron").unwrap()).unwrap();
    let res = config.build();
    let _ = fs::create_dir("output");
    // if fs::read_to_string("output/index.html").unwrap_or_default() != res.html {
    //     // avoid reloading page if html didn't change.
    fs::write("output/style.css", res.css).unwrap();
    fs::write("output/index.html", res.html).unwrap();
    // }
    fs::write("output/ao3.html", res.ao3_html).unwrap();
    fs::write("output/ao3.css", res.ao3_css).unwrap();
}
