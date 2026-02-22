use std::{cell::RefCell, collections::HashMap, fs::read_to_string, path::Path};

use serde::Deserialize;

use crate::config::{
    HashedExt, emit_div, emit_img, emit_p,
    internet_explorer::{Adverts, PathExt, Site, read_page},
};

#[derive(Debug, Clone, Deserialize)]
struct User {
    name: String,
    pic: Option<String>,
    id: u64,
}
impl User {
    fn path(&self) -> String {
        format!("users/{}", self.name.hashed())
    }
    fn realpic(&self) -> &str {
        assert!(self.pic.is_none() || self.pic.as_deref().unwrap().len() > 0);
        self.pic.as_deref().unwrap_or("@img:profiles/guest.png")
    }
}

#[derive(Debug, Clone, Deserialize)]
struct Comment {
    user: u64,
    text: String,
}

#[derive(Debug, Clone, Deserialize)]
struct Chapter {
    start_notes: Option<String>,
    end_notes: Option<String>,
    text: String,
    comments: Vec<Comment>,
}

#[derive(Debug, Clone, Deserialize, enum_as_inner::EnumAsInner)]
enum Category {
    Book,
    Movie,
    Comic,
    Other,
}

#[derive(Debug, Clone, Deserialize)]
struct Fic {
    title: String,
    authorid: u64,
    date: String,
    fandom: String,
    tags: Vec<String>,
    summary: String,
    language: String,
    #[serde(skip_serializing)]
    word_count: RefCell<Option<u32>>,
    chapters: Vec<Chapter>,
    views: u32,
    category: Category,
}
impl Fic {
    fn word_count(&self) -> u32 {
        let mut word_count = self.word_count.borrow_mut();
        if word_count.is_some() {
            return word_count.unwrap();
        }
        *word_count = Some(
            self.chapters
                .iter()
                .map(|x| x.text.split_whitespace().count() as u32)
                .sum(),
        );
        word_count.unwrap()
    }
    fn chapter_path(&self, chapter_idx: usize) -> String {
        format!("fics/{}-{}", self.title.hashed(), chapter_idx)
    }
}

#[derive(Debug, Clone, Deserialize)]
struct FfData {
    users: Vec<User>,
    fics: Vec<Fic>,
}
impl FfData {
    fn user(&self, id: u64) -> &User {
        self.users.iter().find(|x| x.id == id).unwrap()
    }
}

fn emit_fic_blurb(fic: &Fic, html: &mut String, css: &mut String, data: &FfData) {
    emit_div(html, "ff-blurb", |html| {
        html.push_str(&format!(
            r##"
            <div class="ff-blurb-upper">
                <div class="ff-blurb-title">
                    <p><a href="/{}">{}</a> by <a href="/{}">{}</a></p>
                    <p class="ff-uline">{}</p>
                </div>
                <div class="ff-blurb-date"><p><sup>{}</sup></p></div>
            </div>"##,
            fic.chapter_path(0),
            fic.title,
            data.user(fic.authorid).path(),
            data.user(fic.authorid).name,
            fic.fandom,
            fic.date
        ));
        emit_div(html, "ff-blurb-middle", |html| {
            emit_div(html, "ff-blurb-tags", |html| {
                for tag in fic.tags.iter() {
                    emit_p(html, "", &tag);
                }
            });
            html.push_str(&format!(
                r##"<div class="ff-blurb-sum">
                    <p>{}</p>
                </div>"##,
                fic.summary
            ));
        });
        html.push_str(&format!(
            r##"<div class="ff-blurb-lower">
                <p>Language: {}</p>
                <p>Words: {}</p>
                <p>Chapters: {}</p>
                <p>Views: {}</p>
            </div>"##,
            fic.language,
            fic.word_count(),
            fic.chapters.len(),
            fic.views
        ));
    });
}

fn emit_chapter(fic: &Fic, chapter_idx: usize, html: &mut String, css: &mut String, data: &FfData) {
    let chapter = &fic.chapters[chapter_idx];
    if fic.chapters.len() > 1 {
        html.push_str(&format!(
            "<title>{} - Chapter {} - FanFactions - Where Factions Form</title>",
            fic.title,
            chapter_idx + 1
        ));
    } else {
        html.push_str(&format!(
            "<title>{} - FanFactions - Where Factions Form</title>",
            fic.title,
        ));
    }
    emit_div(html, "body ff-fic", |html| {
        emit_fic_blurb(fic, html, css, data);
        if let Some(sn) = &chapter.start_notes {
            emit_div(html, "ff-fic-start", |html| {
                html.push_str(&format!("<h2>Author's Note</h2><p><i>{}</i></p>", sn));
            });
        }
        emit_div(html, "ff-fic-body", |html| {
            html.push_str(&format!("<h1>{}</h1>", fic.title,));
            if fic.chapters.len() > 1 {
                html.push_str(&format!(
                    "<h2>Chapter {}</h2>",
                    chapter_idx + 1,
                    // chapter.text
                ));
            }
            html.push_str(&chapter.text);
        });
        if let Some(en) = &chapter.end_notes {
            emit_div(html, "ff-fic-start", |html| {
                html.push_str(&format!("<h2>End Note</h2><p><i>{}</i></p>", en));
            });
        }
        emit_div(html, "ff-fic-buts", |html| {
            if fic.chapters.len() > chapter_idx + 1 {
                emit_p(
                    html,
                    "ff-button",
                    &format!(
                        r##"<a href="/{}">Next Chapter</a>"##,
                        fic.chapter_path(chapter_idx + 1)
                    ),
                );
            }
            if 0 < chapter_idx {
                emit_p(
                    html,
                    "ff-button",
                    &format!(
                        r##"<a href="/{}">Previous Chapter</a>"##,
                        fic.chapter_path(chapter_idx - 1)
                    ),
                );
            }
        });
        emit_div(html, "ff-fic-comments", |html| {
            html.push_str(&format!("<h2>Comments ({})</h2>", chapter.comments.len()));
            emit_p(html, "warning", "YOU ARE BANNED FROM POSTING COMMENTS");
            for comment in &chapter.comments {
                let user = data.user(comment.user);
                emit_div(html, "ff-comment", |html| {
                    emit_div(html, "ff-comment-top", |html| {
                        emit_img(html, "ff-comment-logo", user.realpic());
                        emit_p(
                            html,
                            "ff-comment-name",
                            &format!(r##"<a href="/{}">{}</a>"##, user.path(), user.name),
                        );
                    });
                    emit_div(html, "ff-comment-text", |html| {
                        emit_p(html, "", &comment.text);
                    });
                });
            }
        });
    });
}

fn emit_profile(user: &User, html: &mut String, css: &mut String, data: &FfData) {
    html.push_str(&format!(
        "<title>{}'s Profile - FanFactions - Where Factions Form</title>",
        user.name
    ));
    emit_div(html, "body ff-profile", |html| {
        emit_div(html, "ff-profile-info", |html| {
            emit_img(html, "ff-profile-logo", user.realpic());
            emit_p(html, "ff-profile-name", &user.name);
        });
        emit_div(html, "ff-profile-fics", |html| {
            html.push_str(&format!("<h1>{}'s Works</h1>", user.name));
            for fic in &data.fics {
                if fic.authorid == user.id {
                    emit_fic_blurb(&fic, html, css, &data);
                }
            }
        });
    });
}

pub fn generate_fanfactions_net(path: &Path, ads: &Adverts) -> Site {
    let data: FfData = toml::from_str(&read_to_string(path.join("data.toml")).unwrap()).unwrap();
    let mut global_css = read_to_string(path.join("style.css")).unwrap();
    let mut pages = HashMap::new();
    let domain: String = "fanfactions.net".into();
    let header_htmlllll = read_to_string(path.join("_header.html")).unwrap();
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
            let mut html = read_to_string(filepath).unwrap();
            // let mut html = html.replace("@header@", &header_html);
            {
                if html.contains("@@books@@") {
                    let mut books = String::new();
                    for fic in &data.fics {
                        if fic.category.is_book() {
                            emit_fic_blurb(&fic, &mut books, &mut global_css, &data);
                        }
                    }
                    html = html.replace("@@books@@", &books);
                }
                if html.contains("@@movies@@") {
                    let mut books = String::new();
                    for fic in &data.fics {
                        if fic.category.is_comic() {
                            emit_fic_blurb(&fic, &mut books, &mut global_css, &data);
                        }
                    }
                    html = html.replace("@@movies@@", &books);
                }
                if html.contains("@@comics@@") {
                    let mut books = String::new();
                    for fic in &data.fics {
                        if fic.category.is_movie() {
                            emit_fic_blurb(&fic, &mut books, &mut global_css, &data);
                        }
                    }
                    html = html.replace("@@comics@@", &books);
                }
                if html.contains("@@others@@") {
                    let mut books = String::new();
                    for fic in &data.fics {
                        if fic.category.is_other() {
                            emit_fic_blurb(&fic, &mut books, &mut global_css, &data);
                        }
                    }
                    html = html.replace("@@others@@", &books);
                }
                if html.contains("@@featured@@") {
                    let mut featured = String::new();
                    for fic in &data.fics {
                        if fic.views > 1000 {
                            emit_fic_blurb(&fic, &mut featured, &mut global_css, &data);
                        }
                    }
                    html = html.replace("@@featured@@", &featured);
                }
            }
            let page = read_page(&domain, html, &page_path, ads, &mut global_css);
            pages.insert(page_path, page);
        }
        //todo!()
    });
    for fic in &data.fics {
        for (i, _) in fic.chapters.iter().enumerate() {
            let mut html = String::new();
            emit_chapter(&fic, i, &mut html, &mut global_css, &data);
            // let html = html.replace("@header@", &header_html);
            pages.insert(
                fic.chapter_path(i),
                read_page(&domain, html, &fic.chapter_path(i), ads, &mut global_css),
            );
        }
    }
    for user in &data.users {
        let mut html = String::new();
        emit_profile(user, &mut html, &mut global_css, &data);
        // let html = html.replace("@header@", &header_html);
        pages.insert(
            user.path(),
            read_page(&domain, html, &user.path(), ads, &mut global_css),
        );
    }
    // shitty hack to make urls work for the header!
    let header_htmlllll =
        read_page(&domain, header_htmlllll, "header", ads, &mut String::new()).html;
    Site {
        header: Some(header_htmlllll),
        domain,
        pages,
        global_css,
    }
}
