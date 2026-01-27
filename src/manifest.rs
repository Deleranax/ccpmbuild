use crate::{Error, Result};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct RepositoryManifest {
    pub name: String,
    pub url: String,
    pub priority: i32,
}

#[derive(Serialize, Deserialize)]
pub struct PackageBase {
    pub description: String,
    pub license: String,
    pub authors: Vec<String>,
    pub maintainers: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct PackageManifest {
    #[serde(flatten)]
    pub base: PackageBase,
    pub version: String,
    pub dependencies: Vec<String>,
}

impl PackageManifest {
    pub fn verify(&self) -> Result<()> {
        if self.base.authors.is_empty() {
            return Err(Error::MissingAuthors);
        }

        if self.base.maintainers.is_empty() {
            return Err(Error::MissingMaintainers);
        }

        spdx::Expression::parse(&self.base.license).map_err(Error::InvalidLicenseId)?;

        Ok(())
    }
}
