use darklua_core::generator::{DenseLuaGenerator, LuaGenerator};
use darklua_core::rules::{ContextBuilder, RenameVariables, Rule};
use darklua_core::{Parser, Resources};

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

/// Minify lua code.
pub fn minify(source: &str) -> anyhow::Result<String> {
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

    // Generate the final source code
    let mut generator = DenseLuaGenerator::new(usize::MAX);
    generator.write_block(&block);
    Ok(generator.into_string())
}
