use std::{path::Path, process};

use config::Config;
use log::{debug, error};
use mdbook::book::{load_book, Book};
use mdbook::errors::Error;
use mdbook::preprocess::{Preprocessor, PreprocessorContext};
use sha2::{Digest, Sha256};

use crate::parser::DocStructure;

pub mod config;
pub mod parser;

const VERSION: &str = env!("CARGO_PKG_VERSION");

/// A no-op preprocessor.
pub struct AutoSummary;

impl AutoSummary {
    pub fn new() -> Self {
        Self
    }

    /// Create an sha256 hash of the existing src/SUMMARY.md file, if one exists
    fn hash_summary(&self, src: &Path) -> Option<Vec<u8>> {
        let summary_path = src.join("SUMMARY.md");

        if !summary_path.is_file() {
            return None;
        }
        let mut hasher = Sha256::new();
        let lines = std::fs::read_to_string(&summary_path).unwrap();
        hasher.update(lines);

        Some(hasher.finalize().to_vec())
    }

    /// Generate a new summary based on the file structure
    fn gen_summary(&self, src: &Path, config: &Config) -> String {
        let Some(mut doc) = DocStructure::from_src(src, config) else {
            error!(
                "Could not find an '{1}' file at '{0}'\nAn '{1}' file must exist at '{0}' when using the autosummary preprocessor!",
                src.display(),
                config.index_name
            );
            process::exit(1);
        };
        doc.relative_to(src)
            .expect("Failed to turn absolute paths into relative");

        let mut gen = doc.to_string().trim_end().to_string();
        gen.push('\n');
        gen.insert_str(
            0,
            &format!(
                "<!-- Generated by mdbook-autosummary v{} - do not edit manually! -->\n\n",
                VERSION
            ),
        );
        gen
    }
}

impl Preprocessor for AutoSummary {
    fn name(&self) -> &str {
        "autosummary"
    }

    fn run(&self, ctx: &PreprocessorContext, book: Book) -> Result<Book, Error> {
        let src_path = ctx.root.join(&ctx.config.book.src);
        let config = Config::from_mdbook(&ctx.config);

        let generated = self.gen_summary(src_path.as_path(), &config);

        let mut hasher = Sha256::new();
        hasher.update(&generated);

        let gen_hash: Vec<u8> = hasher.finalize().to_vec();
        let existing_hash = self.hash_summary(src_path.as_path());

        if existing_hash.is_some() && existing_hash.unwrap() == gen_hash {
            debug!("Generated SUMMARY.md matches existing SUMMARY.md, skipping generation");
            return Ok(book);
        } else {
            std::fs::write(src_path.join("SUMMARY.md"), generated)?;
        }
        let mut conf = ctx.config.build.clone();
        conf.create_missing = false;

        load_book(src_path, &conf)
    }
}

impl Default for AutoSummary {
    fn default() -> Self {
        Self::new()
    }
}
