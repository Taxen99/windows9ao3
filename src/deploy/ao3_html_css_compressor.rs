use crate::config::internet_explorer::SelectionExt;
use std::{collections::HashMap, ops::Deref};

const VALID: &[char] = &[
    'a', 'b', 'c', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't',
    'u', 'v', 'w', 'x', 'y', 'z', 'A', 'B', 'C', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N',
    'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
];

pub fn gen_strings(num: usize) -> Vec<String> {
    if num <= 1 {
        return VALID.iter().map(|x| x.to_string()).collect();
    }
    gen_strings(num - 1)
        .iter()
        .flat_map(|x| {
            VALID.iter().map(|v| {
                let mut x = x.clone();
                x.push(*v);
                x
            })
        })
        .collect()
}

// pub fn gen_a_few_valid_single_ao3_class_names(num: usize) -> Vec<String> {
//     let mut strs = Vec::new();
//     let mut prev: Vec<String> = Vec::new();
//     let mut i = 0;
//     loop {
//         for i in b'a'..=b'z' {
//             let s = prev
//         }
//         strs.push();
//     }
//     let mut strs = Vec::new();
//     for code in 0x7F..=0x10FFFF {
//         if let Some(ch) = std::char::from_u32(code) {
//             // TODO: fix
//             if ch.is_control() || ch.is_whitespace() {
//                 continue;
//             }
//             // dbg!(ch);
//             chars.push(ch);
//         }
//     }
//     chars
// }

pub fn compress_html_css_for_ao3(html: String, mut css: String) -> (String, String) {
    let doc = nipper::Document::from(&html);
    let mut class_occrs: HashMap<String, i32> = HashMap::new();
    for element in doc.select("*").iter() {
        let class_list = element
            .attr("class")
            .map(|x| x.deref().to_owned())
            .unwrap_or_default();
        let classes: Vec<String> = class_list
            .split_ascii_whitespace()
            .map(|x| x.trim().to_owned())
            .collect();
        for class in classes {
            *class_occrs.entry(class.clone()).or_insert(0) += 1;
        }
    }
    let mut classes_sorted = class_occrs.iter().collect::<Vec<_>>();
    classes_sorted.sort_by_key(|(_, n)| -(**n as i32));
    let classes_sorted: Vec<_> = classes_sorted.iter().map(|x| x.0).collect();
    dbg!(classes_sorted);
    let mut class_map = HashMap::new();
    let mut class_names_iter = gen_strings(3).into_iter();

    /////////

    for mut element in doc.select("*").iter() {
        let class_list = element
            .attr("class")
            .map(|x| x.deref().to_owned())
            .unwrap_or_default();
        let classes: Vec<String> = class_list
            .split_ascii_whitespace()
            .map(|x| x.trim().to_owned())
            .collect();
        for class in classes {
            let new_class = class_map
                .entry(class.clone())
                .or_insert_with(|| class_names_iter.next().unwrap().to_string())
                .as_str();
            assert!(element.has_class(&class));
            element.remove_class(&class);
            dbg!(new_class, new_class.len());
            assert!(new_class.len() <= 3);
            element.add_class(new_class);
        }
        if let Some(class_list) = element.attr("class") {
            element.set_attr("class", class_list.to_string().trim());
        }
    }

    // TODO: this is shit!
    let mut class_map: Vec<(String, String)> = class_map.into_iter().collect();
    // quick and dirty solution to avoid replacing the beginning of a larger class name
    class_map.sort_by_key(|(old, _)| -(old.len() as i32));
    for (old, new) in class_map {
        css = css.replace(&format!(".{old}"), &format!(".{new}"));
    }
    let new_html = Into::<String>::into(doc.select("body").html())
        .strip_prefix("<body>")
        .unwrap()
        .strip_suffix("</body>")
        .unwrap()
        .into();
    (new_html, css)
}

pub fn remove_to_much_whitespace(html: String, css: String) -> (String, String) {
    let doc = nipper::Document::from(&html);
    let mut patches = Vec::new();
    for mut element in doc.select("[preservewhitespace]").iter() {
        element.add_class(&format!("PATCH__{}", patches.len()));
        patches.push(
            element
                .text()
                .to_string()
                .replace("<", "&lt;")
                .replace(">", "&gt;"),
        );
        element.attr("preservewhitespace").expect("wat");
        element.remove_attr("preservewhitespace");
        element.set_html("");
    }
    let html = doc.select("body").inner_html();
    let mut html: String = html.split_ascii_whitespace().collect::<Vec<_>>().join(" ");
    // let doc = nipper::Document::from(&html);
    dbg!(&patches);
    // panic!();
    for (i, p) in patches.iter().enumerate() {
        html = html.replace(
            &format!(r#"<p class="PATCH__{}"></p>"#, i),
            &format!("<p>{}</p>", &p),
        );
    }
    // for mut element in doc.select(".np-view > p").iter() {
    //     element.cla(&format!("PATCH__{}", contents.len()));
    //     contents.push(element.text());
    // }
    let css = css
        .replace("\n", " ")
        .split_ascii_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    (html, css)
}
