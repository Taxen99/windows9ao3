use std::path::{Path, PathBuf};

use crate::config::BuildResult;

// pub struct ResolveOptions {
//     pub is_dev: bool,
// }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResouceKind {
    // having icon and img be seperate is kinda shitty, but idk...
    Icon,
    Img,
    Audio,
}
impl ResouceKind {
    // pub fn folder_name(self) -> &'static str {
    //     match self {
    //         ResouceKind::Icon => "icons",
    //         ResouceKind::Img => "img",
    //         ResouceKind::Audio => "audio",
    //     }
    // }
}

pub fn get_resource_path(kind: ResouceKind, name: &str) -> PathBuf {
    let (folder, file_name) = match kind {
        ResouceKind::Icon => ("icons", format!("{name}-9.png")),
        ResouceKind::Img => ("img", format!("{name}")),
        ResouceKind::Audio => ("audio", format!("{name}")),
    };
    Path::new(&format!("res/{folder}/{file_name}")).into()
}

pub fn parse_resouce(res: &str) -> (ResouceKind, &str) {
    let (kind, name) = res
        .split_once(":")
        .expect(&format!("invalid resouce format '{res}'"));
    let kind = match kind {
        "icon" => ResouceKind::Icon,
        "audio" => ResouceKind::Audio,
        "img" => ResouceKind::Img,
        _ => panic!("invalid resource type '{kind}'"),
    };
    (kind, name)
}

fn resolve_for(
    string: &mut String,
    search: &str,
    resolver: &mut impl FnMut(ResouceKind, &str) -> String,
) {
    let mut i = 0;
    assert!(search.ends_with("@"));
    loop {
        match string[i..].find(search) {
            Some(index) => {
                i += index + search.len();
                // dbg!(&string[i..i + 100]);
                let string_slice = &string[i..];
                let end = string_slice.find("\"").expect("unclosed quote");
                let resource = dbg!(parse_resouce(dbg!(&string_slice[..end])));
                //                      vvv -1 because of the @
                string.replace_range((i - 1)..(i + end), &resolver(resource.0, resource.1));
                i += end;
            }
            None => break,
        }
    }
}

pub fn resolve_resources(
    build_result: &BuildResult,
    // opt: ResolveOptions,
    mut resolver: impl FnMut(ResouceKind, &str) -> String,
) -> BuildResult {
    let (mut html, mut css) = (build_result.html.clone(), build_result.css.clone());
    resolve_for(&mut html, "src=\"@", &mut resolver);
    resolve_for(&mut css, "url(\"@", &mut resolver);
    BuildResult { html, css }
}
