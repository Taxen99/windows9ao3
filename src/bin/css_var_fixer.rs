use std::{collections::HashMap, fs::read_to_string};

use arcthing::css_var_remove::css_var_remove;

pub fn main() {
    let css_file = read_to_string("baz/index.css").unwrap();
    let modified_css_file = css_var_remove(&css_file);
    println!("{}", modified_css_file);
}
