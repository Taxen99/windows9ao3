use std::{
    collections::HashMap,
    fs,
    hash::{DefaultHasher, Hash, Hasher},
    path::Path,
};

use crate::{
    config::{BuildResult, HashedExt},
    deploy::{ao3_html_css_compressor::compress_html_css_for_ao3, css_var_remove::css_var_remove},
    resource_resolver::{ResouceKind, get_resource_path, resolve_resources},
};

mod ao3_html_css_compressor;
mod css_var_remove;

const AO3_GIT_PATH: &str = "ao3-git";

// fn do_shit_for_res_type(
//     resource_kind: ResouceKind,
//     map: &mut HashMap<(ResouceKind, String), String>,
// ) {
//     let dir_name = resource_kind.folder_name();
//     for entry in Path::new("res").join(dir_name).read_dir().unwrap() {
//         let entry = entry.unwrap();
//         if entry.file_type().unwrap().is_file() {
//             // let name = match resource_kind {
//             //     ResouceKind::Icon => {
//             //         let name = entry
//             //             .path()
//             //             .with_extension("")
//             //             .file_name()
//             //             .unwrap()
//             //             .to_str()
//             //             .unwrap()
//             //             .to_owned();
//             //         if name.ends_with("-9") {
//             //             continue;
//             //         }
//             //         name
//             //     }
//             //     ResouceKind::Audio => entry
//             //         .path()
//             //         .with_extension("")
//             //         .file_name()
//             //         .unwrap()
//             //         .to_str()
//             //         .unwrap()
//             //         .to_owned(),
//             // };
//             let hash = (resource_kind, entry.path()).hashed();
//             let ext = entry.path().extension().unwrap().to_owned();
//             let target_path = Path::new(AO3_GIT_PATH)
//                 .join(hash.to_string())
//                 .with_extension(ext);
//             if !target_path.exists() {
//                 fs::copy(entry.path(), &target_path).unwrap();
//                 println!("created {}", target_path.to_str().unwrap());
//             }
//             map.insert(
//                 (resource_kind, name),
//                 target_path.to_str().unwrap().to_owned(),
//             );
//         }
//     }
// }
fn do_shit_for_res_type(resource_kind: ResouceKind, name: &str) -> String {
    let dir_name = resource_kind.folder_name();
    let path = Path::new(&get_resource_path(resource_kind, &name)).to_owned();
    let hash = (resource_kind, &path).hashed();
    let ext = path.extension().unwrap().to_owned();
    let target_path = Path::new(AO3_GIT_PATH)
        .join(hash.to_string())
        .with_extension(ext);
    if !target_path.exists() {
        fs::copy(path, &target_path).unwrap();
        println!("created {}", target_path.to_str().unwrap());
    }
    format!(
        "https://taxen99.github.io/wao3/{}",
        target_path.to_str().unwrap()
    )
}

pub fn deploy(build_result: &BuildResult) {
    let css = css_var_remove(&build_result.css);
    let (html, css) = compress_html_css_for_ao3(build_result.html.clone(), css);
    let res = resolve_resources(&BuildResult { html, css }, |kind, res| {
        do_shit_for_res_type(kind, res)
    });
}
