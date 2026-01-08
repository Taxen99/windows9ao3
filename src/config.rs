use std::{
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct App {
    pub name: String,
    pub icon: String,
    pub add_to_desktop: Option<(u8, u8)>,
    pub content: String,
}

impl App {
    pub fn id(&self) -> u64 {
        let mut s = DefaultHasher::new();
        self.name.hash(&mut s);
        s.finish()
    }
}

#[derive(Serialize, Deserialize)]
pub struct FileSystem {
    pub root: Folder,
}

#[derive(Serialize, Deserialize)]
pub enum FsEntry {
    File(File),
    Folder(Folder),
}

#[derive(Serialize, Deserialize)]
pub struct Folder {
    pub content: HashMap<String, FsEntry>,
}
#[derive(Serialize, Deserialize)]
pub enum FileKind {
    App,
}
#[derive(Serialize, Deserialize)]
pub struct File {
    pub kind: FileKind,
    pub link: String,
    pub offset: Option<(u8, u8)>,
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub apps: Vec<App>,
    pub fs: FileSystem,
}
