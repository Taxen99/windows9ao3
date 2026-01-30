use crate::config::HashedExt;
use std::{collections::HashMap, fs, path::Path};

pub struct Page {
    pub html: String,
    pub title: String,
}
pub struct Site {
    pub domain: String,
    pub pages: HashMap<String, Page>,
    pub global_css: String,
}

pub fn read_sites() -> HashMap<String, Site> {
    let mut sites = HashMap::new();
    for entry in Path::new("res/sites").read_dir().unwrap() {
        let entry = entry.unwrap();
        if entry.path().is_dir() {
            let site = read_site(&entry.path());
            let domain = site.domain.clone();
            if let Some(_) = sites.insert(domain.clone(), site) {
                panic!("duplicate site domain {}", domain);
            }
        }
    }
    sites
}

fn read_site(path: &Path) -> Site {
    let domain = path.file_name().unwrap().to_str().unwrap().to_owned();
    let global_css = fs::read_to_string(path.join("style.css")).unwrap();
    let mut pages = HashMap::new();
    for entry in path.read_dir().unwrap() {
        let entry = entry.unwrap();
        if entry.path().is_file() && entry.path().extension().unwrap().to_str().unwrap() == "html" {
            let mut page_path = entry.path();
            page_path.set_extension("");
            let mut page_path = page_path.file_name().unwrap().to_str().unwrap();
            if page_path == "index" {
                page_path = "";
            }
            let html = fs::read_to_string(entry.path()).unwrap();
            let page = read_page(&domain, html);
            pages.insert(page_path.to_owned(), page);
        }
    }
    Site {
        domain,
        pages,
        global_css,
    }
}

fn read_page(domain: &str, html: String) -> Page {
    let doc = nipper::Document::from(&html);
    for mut anchor in doc.select("a").iter() {
        let href = anchor.attr("href").expect("a without href!");
        let loc = if href.starts_with("/") {
            (domain, href.strip_prefix("/").unwrap())
        } else {
            href.split_once("/").unwrap()
        };
        anchor.remove_attr("href");
        dbg!(
            format!("{}-{}", loc.0, loc.1),
            format!("{}-{}", loc.0, loc.1).hashed()
        );
        let id = dbg!(format!("{}-{}", loc.0, loc.1).hashed());
        anchor.add_class(&format!("history-trigger-{} history-trigger", id));
    }
    Page {
        // TODO: shit!
        html: (Into::<String>::into(doc.select("body").html())
            .strip_prefix("<body>")
            .unwrap()
            .strip_suffix("</body>")
            .unwrap()
            .into()),
        title: "Test Title".into(),
    }
}
