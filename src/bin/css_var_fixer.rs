use std::{collections::HashMap, fs::read_to_string};

pub fn main() {
    let css_file = read_to_string("baz/index.css").unwrap();
    let var_decls = css_file
        .lines()
        .map(|x| x.trim())
        .enumerate()
        .filter(|(_, x)| x.starts_with("--"));
    let mut vars = HashMap::new();
    let mut lines_to_be_removed = Vec::new();
    for (i, vd) in var_decls {
        lines_to_be_removed.push(i);
        let vd = vd.trim_start_matches("--");
        let vd = vd.trim_end_matches(";");
        let (name, value) = vd.split_once(':').unwrap();
        let name = name.trim();
        let value = value.trim();
        if vars.insert(name, value).is_some() {
            panic!("duplicate var name");
        }
    }
    let mut modified_css_file = css_file.clone();
    for (name, value) in vars {
        modified_css_file = modified_css_file.replace(format!("var(--{name})").as_str(), value);
    }
    modified_css_file = modified_css_file.replace("calc(25px / 3)", "8.33333px");
    let modified_css_file = modified_css_file
        .lines()
        .enumerate()
        .filter_map(|(i, x)| {
            if lines_to_be_removed.contains(&i) {
                None
            } else {
                Some(x)
            }
        })
        .collect::<Vec<&str>>()
        .join("\n");
    println!("{}", modified_css_file);
}
