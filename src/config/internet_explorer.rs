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
    // a simple html parser...
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
        // anchor.a
        // let new_node = NodeRef::new_element(QualName { prefix: None, ns: ns!(html), local: local_name!("div") }, attributes)
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

// use std::collections::HashMap;

// pub struct Page {
//     pub title: String,
//     pub html: String,
//     pub css: String,
// }

// impl Page {
//     fn new() -> Page {
//         Self {
//             title: String::new(),
//             html: String::new(),
//             css: String::new(),
//         }
//     }
// }

// pub fn parse_page(input: &str) -> Page {
//     let mut page = Page::new();
//     let mut lines = input.lines().peekable();
//     while let Some(line) = lines.next() {
//         if let Some(section_name) = line.strip_prefix("@!") {
//             let section_name = section_name.trim();
//             if !section_name.chars().all(|x| x.is_alphabetic()) {
//                 panic!("invalid section name '{}'", section_name);
//             }
//             let mut section_content = String::new();
//             while let Some(x) = lines.peek() {
//                 if x.starts_with("@!") {
//                     break;
//                 }
//                 section_content.push_str(lines.next().unwrap());
//                 section_content.push('\n');
//             }
//             let section_content = section_content.trim();
//             match section_name {
//                 "title" => page.title = section_content.to_owned(),
//                 "content" => page.html = section_content.to_owned(),
//                 "style" => {
//                     let section_content = section_content
//                         .strip_prefix("<style>")
//                         .expect("expected <style>")
//                         .strip_suffix("</style>")
//                         .expect("expected </style>");
//                     page.css = section_content.to_owned()
//                 }
//                 _ => panic!("invalid section '{}'", section_name),
//             }
//         }
//     }
//     page
// }

// pub struct PageParser {
//     page: Page,
//     input: Vec<char>,
//     idx: usize,
// }

// impl PageParser {
//     pub fn new(text: &str) -> Self {
//         Self {
//             page: Page::new(),
//             input: text.chars().collect(),
//             idx: 0,
//         }
//     }
//     pub fn parse(mut self) -> Page {
//         let mut sections: HashMap<String, String> = HashMap::new();
//         loop {
//             match self.consume() {
//                 Some(c) => match c {
//                     '@' => {
//                         if self.consume_if(|c| c == '!').is_some() {
//                             self.consume_whitespace();
//                             let section_name = self
//                                 .consume_while(|c| !c.is_whitespace())
//                                 .expect("section must have name");
//                             if !section_name.chars().all(|x| x.is_alphabetic()) {
//                                 panic!("invalid section name '{}'", section_name);
//                             }
//                             let section_content = self.parse_section_content();
//                             if let Some(_) = sections.insert(section_name.clone(), section_content)
//                             {
//                                 panic!("duplicacte section '{}'", section_name);
//                             }
//                         }
//                     }
//                     _ => {
//                         panic!("expected section");
//                     }
//                 },
//                 None => {
//                     break;
//                 }
//             }
//         }
//         Page {
//             title: sections.remove("title").unwrap(),
//             html: sections.remove("content").unwrap(),
//             css: sections.remove("style").unwrap(),
//         }
//     }
//     fn parse_section_content(&mut self) -> String {
//         let mut content = String::new();
//         loop {
//             match self.consume() {
//                 Some(c) => match c {
//                     '@' => {
//                         if self.consume_if(|c| c == '!').is_some() {
//                             self.idx -= 2;
//                             break;
//                         }
//                         content.push(c);
//                     }
//                     _ => {
//                         content.push(c);
//                     }
//                 },
//                 None => {
//                     break;
//                 }
//             }
//         }
//         content.trim().to_owned()
//     }
//     fn try_peek(&self) -> Option<char> {
//         self.input.get(self.idx).cloned()
//     }
//     fn consume(&mut self) -> Option<char> {
//         let c = self.try_peek()?;
//         self.idx += 1;
//         Some(c)
//     }
//     fn consume_if(&mut self, filter: impl Fn(char) -> bool) -> Option<char> {
//         if let Some(c) = self.try_peek()
//             && filter(c)
//         {
//             self.idx += 1;
//             return Some(c);
//         }
//         None
//     }
//     fn consume_while(&mut self, filter: impl Fn(char) -> bool) -> Option<String> {
//         let mut res = String::new();
//         while let Some(c) = self.try_peek()
//             && filter(c)
//         {
//             self.idx += 1;
//             res.push(c);
//         }
//         if res.is_empty() {
//             return None;
//         }
//         Some(res)
//     }
//     fn consume_whitespace(&mut self) -> () {
//         self.consume_while(|x| x.is_whitespace());
//     }
//     fn consume_char(&mut self, filter: char) -> bool {
//         if let Some(c) = self.try_peek()
//             && filter == c
//         {
//             self.idx += 1;
//             return true;
//         }
//         false
//     }
// }

// // fn parse_page(text: &str) -> Page {
// //     let chars = text.chars().collect::<Vec<_>>();
// //     let mut page = Page {
// //         title: String::new(),
// //         html: String::new(),
// //         css: String::new(),
// //     };
// //     let mut i = 0;
// //     loop {}
// //     page
// // }
