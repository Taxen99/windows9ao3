use std::{collections::HashMap, ops::Deref};

pub fn gen_a_few_valid_single_char_class_names() -> Vec<char> {
    let mut chars = Vec::new();
    for code in 0x7F..=0x10FFFF {
        if let Some(ch) = std::char::from_u32(code) {
            // TODO: fix
            if ch.is_control() || ch.is_whitespace() {
                continue;
            }
            // dbg!(ch);
            chars.push(ch);
        }
    }
    chars
}

pub fn compress_html_css_for_ao3(html: String, mut css: String) -> (String, String) {
    panic!("this is broken!!");
    let doc = nipper::Document::from(&html);
    let mut class_map: HashMap<String, String> = HashMap::new();
    let mut class_names_iter = gen_a_few_valid_single_char_class_names().into_iter();
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
            assert!(new_class.len() < 3);
            element.add_class(new_class);
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
