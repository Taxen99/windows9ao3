use std::{collections::HashMap, fs::read_to_string, path::Path};

use crate::config::{
    emit_div, emit_p,
    internet_explorer::{Adverts, PathExt, Site, read_page},
};

#[derive(Debug, Clone)]
struct Chapter {
    start_notes: Option<String>,
    end_notes: Option<String>,
    text: String,
}

#[derive(Debug, Clone)]
struct Fic {
    title: String,
    author: String,
    date: String,
    fandom: String,
    tags: Vec<String>,
    summary: String,
    language: String,
    word_count: u32,
    chapters: Vec<Chapter>,
    views: u32,
}

fn emit_fic_blurb(fic: &Fic, html: &mut String, css: &mut String) {
    emit_div(html, "ff-blurb", |html| {
        html.push_str(&format!(
            r##"
            <div class="ff-blurb-upper">
                <div class="ff-blurb-title">
                    <p><a href="#">{}</a> by <a href="#">{}</a></p>
                    <p class="ff-uline">{}</p>
                </div>
                <div class="ff-blurb-date"><p><sup>{}</sup></p></div>
            </div>"##,
            fic.title, fic.author, fic.fandom, fic.date
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
            fic.word_count,
            fic.chapters.len(),
            fic.views
        ));
    });
}

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
