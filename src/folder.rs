use std::fs::{self, read_dir, DirEntry};
use std::io::{self, Error};
use std::path::Path;

use thiserror::Error;

#[derive(thiserror::Error, Debug)]
pub enum FolderError {
    #[error("data store disconnected")]
    OsError(#[from] io::Error),
    #[error("`{0}`")]
    Redaction(String),
    #[error("unknown data store error")]
    Unknown,
}


#[derive(Debug, Clone)]
pub struct FolderNode {
    pub path: String,
    pub expanded: bool,
    pub is_folder: bool,
    pub subfolders: Vec<Box<FolderNode>>,
    pub selected: bool,
}

impl FolderNode {
    pub fn name(&mut self) -> &str {
        let ancestors = Path::new(&self.path).file_name().unwrap().to_str().unwrap();
        return ancestors;
    }
}

pub fn expand(folder: &mut FolderNode) -> Result<(), FolderError> {
    let p: &Path = Path::new(&folder.path);
    let mut entries: Vec<Box<FolderNode>> = read_dir(p)
        .map(|res|  {
            res.map(|e| {
                let de = e.unwrap();
                let file_type = de.file_type().unwrap();
                Box::new(FolderNode {
                        path: String::from(de.path().to_string_lossy()),
                        expanded: false,
                        is_folder: file_type.is_dir(),
                        subfolders: vec![],
                        selected: false
                    })
            })
        })?
        .collect();

        entries.sort_by_key(|a: &Box<FolderNode>| {
            String::from(a.clone().name())
        } );

    folder.subfolders = entries.clone();
    folder.expanded = true;
    Ok(())
}
