use crate::index::Index;
use crate::manifest::PackageManifest;
use crate::package::File;
use crate::{Error, index, package};
use base64::Engine;
use console::{Alignment, pad_str, style};
use sha2::Digest;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::Instant;

const LENGTH: usize = 11;

macro_rules! printfmt {
    ($cat:expr, $msg:expr) => {
        println!(
            "{}: {}",
            style(pad_str($cat, LENGTH, Alignment::Right, None)).bold(),
            $msg
        )
    };
}

/// Build all packages.
pub fn build_all(input: &Path, output: &Path, minify: bool) -> crate::Result<()> {
    let packages_dir = input.join(crate::PACKAGES_ROOT_DIR_NAME);
    let pool_dir = output.join(crate::PACKAGES_POOL_DIR_NAME);
    let index_file = pool_dir.join(crate::INDEX_FILE_NAME);

    println!("Building all packages");
    let start = Instant::now();

    fs::create_dir_all(&pool_dir).map_err(Error::CreateOutputDirectory)?;

    let mut index = if index_file.exists() && index_file.is_file() {
        println!("Index file exists, updating");
        fs::read(&index_file)
            .map_err(Error::ReadIndex)
            .map(|data| serde_json::from_slice(&data))?
            .map_err(Error::DeserializeIndex)?
    } else {
        println!("Index file does not exist, creating a new one");
        Index::new()
    };

    println!();

    let mut queue = Vec::new();

    match fs::read_dir(&packages_dir) {
        Ok(dir) => queue.extend(
            dir.filter_map(Result::ok)
                .filter(|entry| entry.file_type().is_ok_and(|file_type| file_type.is_dir()))
                .map(|entry| entry.path()),
        ),
        Err(e) => return Err(Error::ReadInput(e)),
    };

    let mut current = 0usize;
    let total = queue.len();
    if let Some(e) = queue
        .iter()
        .map(|p| {
            current += 1;
            build(&mut index, &current, &total, p, output, minify)
        })
        .find_map(Result::err)
    {
        return Err(e);
    }

    fs::write(
        index_file,
        serde_json::to_string(&index).map_err(Error::SerializeIndex)?,
    )
    .map_err(Error::WriteIndex)?;

    println!(
        "{} Built {total} package{} in {}ms",
        style("Success!").green(),
        if total > 0 { "s" } else { "" },
        start.elapsed().as_millis()
    );

    Ok(())
}

/// Build a package.
fn build(
    index: &mut Index,
    current: &usize,
    total: &usize,
    input: &Path,
    output: &Path,
    minify: bool,
) -> crate::Result<()> {
    let name = input
        .file_name()
        .and_then(|str| str.to_str())
        .ok_or(Error::InvalidPackageName(input.into()))?;

    println!("{} Package {current}/{total}", style("Building…").blue());

    let manifest = fs::read(input.join(crate::MANIFEST_FILE_NAME))
        .map_err(Error::ReadManifest)
        .map(|bytes| serde_json::from_slice::<PackageManifest>(&bytes))?
        .map_err(Error::ParseManifest)?;

    println!();
    printfmt!("Name", name);
    printfmt!("Description", manifest.base.description);
    printfmt!("License", manifest.base.license);
    manifest
        .base
        .authors
        .iter()
        .for_each(|author| printfmt!("Author", author));
    manifest
        .base
        .maintainers
        .iter()
        .for_each(|maintainer| printfmt!("Maintainer", &maintainer));
    println!();
    printfmt!("Version", manifest.version);
    manifest
        .dependencies
        .iter()
        .for_each(|dep| printfmt!("Dependency", dep));

    // Verify the manifest
    manifest.verify()?;

    let mut package = package::Package {
        manifest,
        files: HashMap::new(),
    };

    // Walk source tree
    let base_source_path = input.join(crate::PACKAGES_SOURCE_DIR_NAME);
    let mut stack = vec![base_source_path.clone()];

    while let Some(src) = stack.pop() {
        if src.is_dir() {
            match fs::read_dir(src) {
                Ok(dir) => stack.extend(dir.filter_map(Result::ok).map(|entry| entry.path())),
                Err(e) => return Err(Error::ReadSourceDir(e)),
            }
        } else {
            let path = src
                .strip_prefix(&base_source_path)
                .map_err(Error::RemoveSourcePrefix)?;

            printfmt!("Source", path.display());

            let mut source = fs::read_to_string(&src).map_err(Error::ReadSource)?;

            if minify {
                source = crate::minify::minify(&source).map_err(Error::MinifySource)?;
            }

            package.files.insert(
                path.to_string_lossy().to_string(),
                File {
                    digest: hex::encode(sha2::Sha256::digest(source.bytes().collect::<Vec<u8>>())),
                    content: source,
                },
            );
        }
    }
    println!();

    let file = format!("{}.{}.ccp", name, package.manifest.version);

    printfmt!("File", file);

    let data = base64::prelude::BASE64_STANDARD.encode(deflate::deflate_bytes(
        serde_json::to_string(&package)
            .map_err(Error::SerializePackage)?
            .as_bytes(),
    ));

    fs::write(output.join(crate::PACKAGES_POOL_DIR_NAME).join(file), &data)
        .map_err(Error::WritePackage)?;

    println!();

    let version = index::Version {
        digest: hex::encode(sha2::Sha256::digest(data)),
        dependencies: package.manifest.dependencies,
    };

    match index.get_mut(name) {
        None => {
            let mut versions = HashMap::new();
            versions.insert(package.manifest.version.clone(), version);

            index.insert(
                name.to_string(),
                index::Package {
                    manifest: package.manifest.base,
                    versions,
                    latest_version: package.manifest.version.clone(),
                },
            );
        }
        Some(p) => {
            p.manifest = package.manifest.base;
            p.latest_version = package.manifest.version.clone();
            p.versions.insert(package.manifest.version.clone(), version);
        }
    }

    Ok(())
}
