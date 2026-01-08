use std::collections::HashMap;

pub fn css_var_remove(css_file: &str) -> String {
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
            panic!("duplicate var name {name}");
        }
    }
    let mut modified_css_file = css_file.to_owned();
    for (name, value) in &vars {
        modified_css_file = modified_css_file.replace(format!("var(--{name})").as_str(), value);
    }
    // TODO: we do this twice for nested var decls. fix!
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
    modified_css_file
}
