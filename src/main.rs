use clap::Parser;
use clap_derive::Subcommand;
use console::style;
use std::path::PathBuf;
use std::process::ExitCode;
use thiserror::Error;

pub mod build;
pub mod index;
pub mod manifest;
pub mod minify;
pub mod package;
pub mod repair;

const MANIFEST_FILE_NAME: &str = "manifest.json";
const PACKAGES_ROOT_DIR_NAME: &str = "packages";
const PACKAGES_SOURCE_DIR_NAME: &str = "source";
const PACKAGES_POOL_DIR_NAME: &str = "pool";
const INDEX_FILE_NAME: &str = "index.json";

#[derive(Error, Debug)]
pub enum Error {
    #[error("Unable to read input directory: {0}")]
    ReadInput(std::io::Error),

    #[error("Invalid package name: {0}")]
    InvalidPackageName(PathBuf),

    #[error("Unable to read package manifest: {0}")]
    ReadManifest(std::io::Error),

    #[error("Missing authors")]
    MissingAuthors,

    #[error("Missing maintainers")]
    MissingMaintainers,

    #[error("Malformed package manifest: {0}")]
    ParseManifest(serde_json::Error),

    #[error("Invalid SPDX license identifier: \n{0}")]
    InvalidLicenseId(spdx::ParseError),

    #[error("Unable to read source directory: {0}")]
    ReadSourceDir(std::io::Error),

    #[error("Invalid source path: {0}")]
    RemoveSourcePrefix(std::path::StripPrefixError),

    #[error("Unable to read source file: {0}")]
    ReadSource(std::io::Error),

    #[error("Unable to minify source file: {0}")]
    MinifySource(anyhow::Error),

    #[error("Unable to serialize package: {0}")]
    SerializePackage(serde_json::Error),

    #[error("Unable to write package: {0}")]
    WritePackage(std::io::Error),

    #[error("Unable to create output directory: {0}")]
    CreateOutputDirectory(std::io::Error),

    #[error("Unable to serialize index: {0}")]
    SerializeIndex(serde_json::Error),

    #[error("Unable to write index: {0}")]
    WriteIndex(std::io::Error),

    #[error("Unable to read index: {0}")]
    ReadIndex(std::io::Error),

    #[error("Malformed index: {0}")]
    DeserializeIndex(serde_json::Error),
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Parser)]
#[command(version, author, about, long_about = None)]
struct CCPMBuild {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
#[command(version, author, about, long_about = None)]
enum Command {
    /// Build the packages into a repository
    Build {
        /// Minify Lua source files
        #[clap(short, long)]
        minify: bool,

        /// Input directory containing the repository manifest and the packages source directory
        #[arg(value_parser = validate_dir_path)]
        input: PathBuf,

        /// Output directory will contain the repository
        #[arg(value_parser = validate_dir_path)]
        output: PathBuf,
    },
    /// Repair the package index
    Repair {
        /// Remove invalid packages detected during repair
        #[clap(short, long)]
        remove_invalid_packages: bool,

        /// Directory containing the repository
        #[clap(value_parser = validate_dir_path)]
        path: PathBuf,
    },
}

/// Validate the directory arguments
fn validate_dir_path(arg: &str) -> std::result::Result<PathBuf, String> {
    match arg.parse::<PathBuf>() {
        Ok(path) => {
            if path.exists() && path.is_dir() {
                Ok(path)
            } else {
                Err("directory does not exist".to_string())
            }
        }
        Err(_) => Err("invalid path".to_string()),
    }
}

fn main() -> ExitCode {
    if let Err(e) = match CCPMBuild::parse().command {
        Command::Build {
            input,
            output,
            minify,
        } => build::build_all(&input, &output, minify),
        Command::Repair { .. } => todo!(),
    } {
        eprintln!("{} {}", style("Error!").red(), e);
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
