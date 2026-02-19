use rand::seq::IndexedRandom;

use crate::config::{FsEntry, HashedExt, internet_explorer::fanfactions::generate_fanfactions_net};
use std::{collections::HashMap, fs, path::Path};

mod fanfactions;

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
            let mut site = match entry.path().file_name().unwrap().to_str().unwrap() {
                "fanfactions.net" => generate_fanfactions_net(&entry.path(), &ads),
                _ => read_site(&entry.path(), &ads),
            };
            site.global_css = site.global_css.replace(
                "@onpageload@",
                ".main:has(:is(.history-item:hover, .ie-tb-refresh:active))",
            );
            let domain = site.domain.clone();
            if let Some(_) = sites.insert(domain.clone(), site) {
                panic!("duplicate site domain {}", domain);
            }
        }
    }
    sites
}

trait PathExt {
    fn read_dir_recurse(&self, cb: &mut dyn FnMut(&Path));
}
impl PathExt for Path {
    fn read_dir_recurse(&self, cb: &mut dyn FnMut(&Path)) {
        for entry in self.read_dir().unwrap() {
            let entry = entry.unwrap();
            if entry.file_type().unwrap().is_dir() {
                entry.path().read_dir_recurse(cb);
            } else {
                assert!(entry.file_type().unwrap().is_file());
                cb(&entry.path());
            }
        }
    }
}

fn read_site(path: &Path, ads: &Adverts) -> Site {
    let domain = path.file_name().unwrap().to_str().unwrap().to_owned();
    let mut global_css = fs::read_to_string(path.join("style.css")).unwrap_or_default();
    let mut pages = HashMap::new();
    // let template = fs::read_to_string(path.join("_template.html")).ok();
    path.read_dir_recurse(&mut |filepath| {
        let mut page_path = filepath.strip_prefix(path).unwrap().to_owned();
        if page_path.extension().unwrap().to_str().unwrap() == "html" {
            page_path.set_extension("");
            let mut page_path = page_path.to_str().unwrap();
            // dbg!(page_path);
            if page_path == "index" {
                page_path = "";
            }
            let page_path = page_path.replace("+", "/");
            let html = fs::read_to_string(filepath).unwrap();
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

trait SelectionExt {
    fn inner_html(&self) -> String;
    fn outer_html(&self) -> String;
}
impl SelectionExt for nipper::Selection<'_> {
    fn inner_html(&self) -> String {
        let mut inner_html = String::new();
        for child in self.children().iter() {
            inner_html.push_str(&child.outer_html());
        }
        inner_html
    }

    fn outer_html(&self) -> String {
        self.html().to_string()
    }
}

fn read_page(domain: &str, html: String, path: &str, ads: &Adverts, css: &mut String) -> Page {
    println!("PATH: {} - {}", path, path.hashed());
    let doc = nipper::Document::from(&html);
    for mut ad in doc.select("advert").iter() {
        let kind = ad.attr("type").expect("advert without type").to_string();
        let marquee = ad.attr("marquee").unwrap_or_default().to_string();
        let pool = match kind.as_ref() {
            "banner" => &ads.banners,
            "box" => &ads.boxes,
            _ => panic!("invalid ad type"),
        };
        let selected_ad = pool.choose(&mut rand::rng()).unwrap();
        match marquee.as_ref() {
            "true" => {
                ad.replace_with_html(format!(
                    r##"
                    <marquee type="imgrepeat">
                    <img src="@ad:{}" />
                    </marquee>
                    "##,
                    selected_ad.src
                ));
            }
            _ => {
                ad.replace_with_html(format!(
                    r##"
                    <div class="advert advert-{}">
                    <img src="@ad:{}" />
                    </div>
                    "##,
                    kind, selected_ad.src
                ));
            }
        }
    }
    for mut marquee in doc.select("marquee").iter() {
        let kind = marquee.attr("type").expect("marquee without type!");
        let replacement_html = match kind.as_ref() {
            "imgrepeat" => {
                let imgelm = marquee.select("img").first();
                let src = imgelm.attr("src").unwrap().to_string();
                let replacement_html = format!(
                    r##"
                    <div class="marquee-repeat">
                        <div class="marquee-img marquee-img-{}"></div>
                    </div>
                "##,
                    src.hashed()
                );
                css.push_str(&format!(
                    r##"
                    .marquee-img-{} {{
                        background-image: url("{}")
                    }}
                "##,
                    src.hashed(),
                    src
                ));
                replacement_html
            }
            _ => {
                let replacement_html = format!(
                    r##"
        <div class="marquee">
            <div class="marquee1">{0}</div>
            <div class="marquee1">{0}</div>
            <div class="marquee1">{0}</div>
            <div class="marquee1">{0}</div>
            <div class="marquee1">{0}</div>
            <div class="marquee1">{0}</div>
            <div class="marquee1">{0}</div>
            <div class="marquee1">{0}</div>
        </div>
        "##,
                    dbg!(marquee.inner_html())
                );
                replacement_html
            }
        };

        marquee.replace_with_html(replacement_html);
    }

    for mut four04 in doc.select("e404").iter() {
        // taken&modified from https://deltarune.com/december/
        let replacement = format!(
            r##"
            <title>HTTP 404 Not Found</title>
<div class="e404">
<h1><img src="@icon:ieerror" alt="Info"> The page cannot be found</h1>
<p>The page you are looking for might have been removed, had its name changed, or is temporarily unavailable.</p>
<hr>
<p>Please try the following:</p>
<ul>
<li>If you typed the page address in the Address bar, make sure that it is spelled correctly.</li>
<li>Open the <a href="{0}">http://{0}</a> home page, and then look for links to the information you want.</li>
<li>Click the <a href="#" class="history-back"><img src="@icon:back" alt="Back"> Back</a> button to try another link.</li>
<li>Click <a href="#"><img src="@icon:search" alt="Search"> Search</a> to look for information on the Internet.</li>
</ul>
<p>HTTP 404 - File not found<br>Internet Explorer</p>
</div>
        "##,
            domain
        );
        four04.replace_with_html(replacement);
    }

    for mut anchor in doc.select("a").iter() {
        let href = anchor.attr("href").expect("a without href!");
        if href.as_ref() == "#" {
            continue;
        }
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
