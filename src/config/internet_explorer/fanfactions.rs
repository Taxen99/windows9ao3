use std::{collections::HashMap, fs::read_to_string, path::Path};

use crate::config::internet_explorer::{Adverts, PathExt, Site, read_page};

pub fn generate_fanfactions_net(path: &Path, ads: &Adverts) -> Site {
    let mut global_css = read_to_string(path.join("style.css")).unwrap();
    let mut pages = HashMap::new();
    let domain: String = "fanfactions.net".into();
    let header_html = read_to_string(path.join("_header.html")).unwrap();
    path.read_dir_recurse(&mut |filepath| {
        let mut page_path = filepath.strip_prefix(path).unwrap().to_owned();
        if page_path.extension().unwrap().to_str().unwrap() == "html"
            && !page_path
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .starts_with("_")
        {
            page_path.set_extension("");
            let mut page_path = page_path.to_str().unwrap().to_owned();
            if page_path == "index" {
                page_path = "".into();
            }
            let html = read_to_string(filepath).unwrap();
            let html = html.replace("@header@", &header_html);
            let page = read_page(&domain, html, &page_path, ads, &mut global_css);
            pages.insert(page_path, page);
        }
        //todo!()
    });
    Site {
        domain,
        pages,
        global_css,
    }
}
