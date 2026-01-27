use crate::manifest::PackageBase;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type Index = HashMap<String, Package>;

#[derive(Serialize, Deserialize)]
pub struct Package {
    #[serde(flatten)]
    pub manifest: PackageBase,
    pub versions: HashMap<String, Version>,
}

#[derive(Serialize, Deserialize)]
pub struct Version {
    pub digest: String,
    pub dependencies: Vec<String>,
}
