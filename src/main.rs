mod ast;
mod languages;
mod processor;
mod rules;

use anyhow::{Context, Result, bail};
use ignore::WalkBuilder;
use languages::SourceLanguage;
use processor::Processor;
use std::path::PathBuf;

fn main() -> Result<()> {
    let mut args = std::env::args_os().skip(1);
    let path = match (args.next(), args.next()) {
        (Some(path), None) => PathBuf::from(path),
        _ => bail!("Usage: uncomment <path>"),
    };
    let root = path
        .canonicalize()
        .with_context(|| format!("Failed to open {}", path.display()))?;
    let walk_root = root.clone();
    let walker = WalkBuilder::new(&root)
        .hidden(false)
        .ignore(false)
        .git_global(false)
        .require_git(false)
        .filter_entry(move |entry| {
            let path = entry.path();
            path.file_name().is_none_or(|name| name != ".git")
                && (path == walk_root || !path.is_dir() || !path.join(".git").exists())
        })
        .build();

    let mut processor = Processor::new();
    let mut processed = 0;
    let mut modified = 0;
    for entry in walker {
        let entry = entry?;
        if !entry.file_type().is_some_and(|kind| kind.is_file()) {
            continue;
        }
        if let Some(language) = SourceLanguage::for_path(entry.path()) {
            if processor.process_file(entry.path(), language)? {
                modified += 1;
            }
            processed += 1;
        }
    }
    println!("Processed {processed} files, modified {modified}.");
    Ok(())
}
