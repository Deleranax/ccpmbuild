use crate::manifest::PackageManifest;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub struct Package {
    #[serde(flatten)]
    pub manifest: PackageManifest,
    pub files: HashMap<String, File>,
}

#[derive(Serialize, Deserialize)]
pub struct File {
    pub content: String,
    pub digest: String,
}
