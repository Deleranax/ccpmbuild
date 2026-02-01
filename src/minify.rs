use darklua_core::generator::{DenseLuaGenerator, LuaGenerator, TokenBasedLuaGenerator};
use darklua_core::rules::{ContextBuilder, RemoveComments, RemoveSpaces, RenameVariables, Rule};
use darklua_core::{Parser, Resources};
use regex::Regex;
use std::sync::LazyLock;

pub const GLOBALS: &[&str] = &[
    "$default",
    "colors",
    "colours",
    "commands",
    "disk",
    "fs",
    "gps",
    "help",
    "http",
    "io",
    "keys",
    "multishell",
    "os",
    "paintutils",
    "parallel",
    "peripheral",
    "pocket",
    "rednet",
    "redstone",
    "settings",
    "shell",
    "term",
    "textutils",
    "turtle",
    "vector",
    "window",
];

// Regex
static NO_MINIFY: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^\s*--!\s*no-minify").unwrap());
static PRESERVE_LINES: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^\s*--!\s*preserve-lines").unwrap());
static PRESERVE_LINES_STRICT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^\s*--!\s*preserve-lines-strict").unwrap());
static EMPTY_LINES: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?m)^\s*").unwrap());

/// Minify lua code.
pub fn minify(source: &str) -> anyhow::Result<String> {
    // Flags
    let no_minify = NO_MINIFY.is_match(source);
    let preserve_lines = PRESERVE_LINES.is_match(source);
    let preserve_lines_strict = PRESERVE_LINES_STRICT.is_match(source);

    // Return if no-minify
    if no_minify {
        return Ok(source.to_string());
    }

    // Parse the current code
    let mut block = Parser::default()
        .preserve_tokens()
        .parse(source)
        .map_err(|e| anyhow::anyhow!(e))?;

    // Context
    let resources = Resources::from_memory();
    let context = ContextBuilder::new(".", &resources, source).build();

    // Rules
    RenameVariables::new(GLOBALS.iter().map(ToString::to_string))
        .process(&mut block, &context)
        .map_err(|e| anyhow::anyhow!(e))?;
    RemoveComments::default()
        .process(&mut block, &context)
        .map_err(|e| anyhow::anyhow!(e))?;
    RemoveSpaces::default()
        .process(&mut block, &context)
        .map_err(|e| anyhow::anyhow!(e))?;

    let mut output = if preserve_lines || preserve_lines_strict {
        let mut generator = Box::new(TokenBasedLuaGenerator::new(source));
        generator.write_block(&block);
        generator.into_string()
    } else {
        let mut generator = Box::new(DenseLuaGenerator::new(usize::MAX));
        generator.write_block(&block);
        generator.into_string()
    };

    if !preserve_lines_strict {
        output = EMPTY_LINES.replace_all(&output, "").to_string();
    }

    Ok(output)
}
