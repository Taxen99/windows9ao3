use std::path::{Path, PathBuf};

use crate::config::BuildResult;

// pub struct ResolveOptions {
//     pub is_dev: bool,
// }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResouceKind {
    Icon,
    Img,
    Audio,
}
impl ResouceKind {
    pub fn folder_name(self) -> &'static str {
        match self {
            ResouceKind::Icon => "icons",
            ResouceKind::Img => "img",
            ResouceKind::Audio => "audio",
        }
    }
}

pub fn get_resource_path(kind: ResouceKind, name: &str) -> PathBuf {
    let (folder, file_name) = match kind {
        ResouceKind::Icon => ("icons", format!("{name}-9.png")),
        ResouceKind::Img => ("img", format!("{name}")),
        ResouceKind::Audio => ("audio", format!("{name}")),
    };
    Path::new(&format!("{folder}/{file_name}")).into()
}

pub fn parse_resouce(res: &str) -> (ResouceKind, &str) {
    let (kind, name) = res
        .split_once(":")
        .expect(&format!("invalid resouce format '{res}'"));
    let kind = match kind {
        "icon" => ResouceKind::Icon,
        "audio" => ResouceKind::Audio,
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
    loop {
        match string[i..].find(search) {
            Some(index) => {
                i = index + search.len();
                let string_slice = &string[i..];
                let end = i + string_slice.find("\"").expect("unclosed quote");
                let resource = parse_resouce(&string_slice[i..end]);
                string.replace_range(i..end, &resolver(resource.0, resource.1));
                i = end;
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
