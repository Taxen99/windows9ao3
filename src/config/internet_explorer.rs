use rand::seq::IndexedRandom;

use crate::config::HashedExt;
use std::{collections::HashMap, fs, path::Path};

pub struct Page {
    pub html: String,
    pub title: String,
    pub url: Url,
}
pub struct Site {
    pub domain: String,
    pub pages: HashMap<String, Page>,
    pub global_css: String,
}

#[derive(Debug)]
pub struct Advert {
    src: String,
}

#[derive(Debug)]
pub struct Adverts {
    boxes: Vec<Advert>,
    banners: Vec<Advert>,
}

pub fn read_adverts() -> Adverts {
    let mut boxes: Vec<Advert> = Vec::new();
    let mut banners: Vec<Advert> = Vec::new();
    for entry in Path::new("res/ads/banner").read_dir().unwrap() {
        let entry = entry.unwrap();
        if entry.path().is_file() {
            let name: String = entry.path().file_name().unwrap().to_str().unwrap().into();
            banners.push(Advert {
                src: format!("banner/{name}"),
            });
        }
    }
    for entry in Path::new("res/ads/box").read_dir().unwrap() {
        let entry = entry.unwrap();
        if entry.path().is_file() {
            let name: String = entry.path().file_name().unwrap().to_str().unwrap().into();
            boxes.push(Advert {
                src: format!("box/{name}"),
            });
        }
    }
    Adverts { boxes, banners }
}

pub fn read_sites() -> HashMap<String, Site> {
    let ads = read_adverts();
    let mut sites = HashMap::new();
    for entry in Path::new("res/sites").read_dir().unwrap() {
        let entry = entry.unwrap();
        if entry.path().is_dir() {
            let site = read_site(&entry.path(), &ads);
            let domain = site.domain.clone();
            if let Some(_) = sites.insert(domain.clone(), site) {
                panic!("duplicate site domain {}", domain);
            }
        }
    }
    sites
}

fn read_site(path: &Path, ads: &Adverts) -> Site {
    let domain = path.file_name().unwrap().to_str().unwrap().to_owned();
    let global_css = fs::read_to_string(path.join("style.css")).unwrap_or_default();
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
            let page_path = page_path.replace("+", "/");
            let html = fs::read_to_string(entry.path()).unwrap();
            let page = read_page(&domain, html, &page_path, ads);
            pages.insert(page_path, page);
        }
    }
    Site {
        domain,
        pages,
        global_css,
    }
}

fn read_page(domain: &str, html: String, path: &str, ads: &Adverts) -> Page {
    let doc = nipper::Document::from(&html);
    for mut ad in doc.select("advert").iter() {
        dbg!(
            Into::<String>::into(doc.select("body").html())
                .strip_prefix("<body>")
                .unwrap()
                .strip_suffix("</body>")
                .unwrap(),
        );
        dbg!(ad.length());
        let kind = ad.attr("type").expect("advert without type").to_string();
        let pool = match kind.as_ref() {
            "banner" => &ads.banners,
            "box" => &ads.boxes,
            _ => panic!("invalid ad type"),
        };
        let selected_ad = pool.choose(&mut rand::rng()).unwrap();
        ad.replace_with_html(format!(
            r##"
            <div class="advert advert-{}">
            <img src="@ad:{}" />
            </div>
            "##,
            kind, selected_ad.src
        ));
        dbg!("FOO, ", kind);
        dbg!(
            Into::<String>::into(doc.select("body").html())
                .strip_prefix("<body>")
                .unwrap()
                .strip_suffix("</body>")
                .unwrap(),
        );
    }
    if domain == "epicdinner.com" {
        dbg!(doc.select("advert").length());
        dbg!(
            Into::<String>::into(doc.select("body").html())
                .strip_prefix("<body>")
                .unwrap()
                .strip_suffix("</body>")
                .unwrap(),
        );
    }

    for mut anchor in doc.select("a").iter() {
        let href = anchor.attr("href").expect("a without href!");
        // let loc = if href.starts_with("/") {
        //     (domain, href.strip_prefix("/").unwrap())
        // } else {
        //     href.split_once("/").unwrap()
        // };
        let url = Url::parse_maybe_local(&href, domain);
        anchor.remove_attr("href");
        // anchor.set_attr("href", &format!("#{}", loc.hashed()));
        anchor.set_attr("href", "#");
        let id = url.hashed();
        anchor.add_class(&format!("history-trigger-{} history-trigger", id));
    }
    let mut title = doc.select("title").text().trim().to_owned();
    if title.is_empty() {
        title = Path::new(domain).join(path).to_str().unwrap().into();
    }
    Page {
        // TODO: shit!
        html: (Into::<String>::into(doc.select("body").html())
            .strip_prefix("<body>")
            .unwrap()
            .strip_suffix("</body>")
            .unwrap()
            .into()),
        title: title,
        url: Url::from_parts(domain, path),
    }
}

#[derive(Debug)]
pub struct Url {
    domain: String,
    path: String,
}
impl Url {
    pub fn parse(url: &str) -> Self {
        let parsed = parse_url(url);
        assert!(!parsed.0.is_empty());
        Self {
            domain: parsed.0,
            path: parsed.1,
        }
    }
    pub fn from_parts(domain: &str, path: &str) -> Self {
        Self {
            domain: domain.into(),
            path: path.into(),
        }
    }
    pub fn parse_maybe_local(url: &str, domain: &str) -> Self {
        let mut parsed = parse_url(url);
        // assert!(parsed.0.is_empty());
        if parsed.0.is_empty() {
            parsed.0 = domain.into();
        }
        Self {
            domain: parsed.0,
            path: parsed.1,
        }
    }
    pub fn hashed(&self) -> u64 {
        format!("{}|{}", self.domain, self.path).hashed()
    }
    pub fn domain(&self) -> &str {
        &self.domain
    }
    pub fn path(&self) -> &str {
        &self.path
    }
    pub fn to_string(&self) -> String {
        format!("{}/{}", self.domain, self.path)
    }
}

fn parse_url(url: &str) -> (String, String) {
    assert!(!url.is_empty());
    assert!(
        !url.starts_with("http") && !url.contains(":"),
        "urls should not contain a scheme"
    );
    if let Some((domain, path)) = url.split_once("/") {
        // assert!(!domain.is_empty(), "must give global url");
        (domain.into(), path.into())
    } else {
        (url.into(), "".into())
    }
}

// pub fn hash_url_parts()
