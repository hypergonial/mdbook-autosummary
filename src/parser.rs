use std::{
    fmt::Display,
    fs::File,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

use anyhow::Error;
use log::warn;
use path_slash::PathExt;
use walkdir::WalkDir;

use crate::Config;

/// Representation of an mdbook src directory entry
/// This may be a folder or a .md file
#[derive(Debug, Clone)]
pub enum DocStructure {
    /// Represents an mdbook src folder
    Folder(DocFolder),
    /// Represents an mdbook src .md file
    File(DocFile),
}

impl DocStructure {
    /// Create a new DocStructure from the given path as root.
    /// This will recursively discover folders and .md files
    pub fn from_src(src: &Path, config: &Config) -> Option<Self> {
        Self::from_path(src, config, 0)
    }

    /// Create a new DocStructure from a path
    /// This will recursively discover folders and .md files
    ///
    /// The depth parameter is used to determine the depth of the folder relative to the src folder
    ///
    /// ### Note:
    ///
    /// If you want to construct a new DocStructure from src, use `DocStructure::from_src` instead.
    pub fn from_path(path: &Path, config: &Config, depth: u16) -> Option<Self> {
        if !path.exists() {
            panic!("Path doesn't exist: '{}'", path.display());
        }

        if path.is_dir() {
            DocFolder::from_path(path, config, depth).map(DocStructure::Folder)
        } else {
            DocFile::from_path(path, config, depth).map(DocStructure::File)
        }
    }

    /// Change all contained paths to be relative to the provided root path
    pub fn relative_to(&mut self, root: &Path) -> Result<(), Error> {
        match self {
            DocStructure::Folder(folder) => {
                folder.path = folder.path.strip_prefix(root)?.to_path_buf();
                for child in &mut folder.children {
                    child.relative_to(root)?;
                }
            }
            DocStructure::File(file) => {
                file.path = file.path.strip_prefix(root)?.to_path_buf();
            }
        }
        Ok(())
    }
}

impl Display for DocStructure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DocStructure::Folder(folder) => write!(f, "{folder}"),
            DocStructure::File(file) => write!(f, "{file}"),
        }
    }
}

/// Representation of a folder inside the mdbook src folder
///
/// Some special rules apply when using mdbook-autosummary:
///
/// - The folder's title is always the title of the index.md file
/// - An index.md file is required for the folder to be included in the summary
#[derive(Debug, Clone)]
pub struct DocFolder {
    /// The extracted title of the folder, should match index.md's first h1 heading
    /// Falls back to the folder's name if the index.md has no h1 heading
    pub title: String,
    /// The path to the folder
    pub path: PathBuf,
    /// Doc items the folder contains
    pub children: Vec<DocStructure>,
    /// The depth of the folder relative to the src folder
    pub depth: u16,
    /// The name of the index file, located at the root of the folder
    pub index: String,
}

impl DocFolder {
    pub fn new(
        title: impl Into<String>,
        path: PathBuf,
        children: &[DocStructure],
        depth: u16,
        index_name: impl Into<String>,
    ) -> Self {
        DocFolder {
            title: title.into(),
            path,
            children: children.to_vec(),
            depth,
            index: index_name.into(),
        }
    }

    /// Create a new DocFolder from a path
    ///
    /// It tries to find the title of the folder by looking for an index.md file.
    /// If an index.md cannot be found at the root of the folder, this function returns None.
    ///
    /// Children will be discovered recursively.
    pub fn from_path(path: &Path, config: &Config, depth: u16) -> Option<Self> {
        if !path.is_dir() {
            return None;
        }

        // Find and parse index.md if it exists for a title
        let index = DocFile::from_path(path.join(&config.index_name).as_path(), config, depth + 1);

        // If the folder has no index.md, ignore it
        let index = index.as_ref()?;

        // If the index.md has no name, fall back to the folder name
        let title = match index.title.as_str() {
            t if t == config.index_name => path
                .file_name()
                .expect("Path has no filename.")
                .to_str()
                .expect("Path isn't valid UTF-8.")
                .to_string(),
            t => t.to_string(),
        };

        let children: Vec<DocStructure> = WalkDir::new(path)
            .sort_by_file_name()
            .min_depth(1)
            .max_depth(1)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.file_name() != config.index_name.as_str() && e.file_name() != "SUMMARY.md"
            })
            .filter(|e| {
                e.file_type().is_dir()
                    || e.file_name()
                        .to_str()
                        .expect("Path isn't valid UTF-8.")
                        .ends_with(".md")
            })
            .filter_map(|e| DocStructure::from_path(e.path(), config, depth + 1))
            .collect();

        Some(DocFolder {
            title,
            path: path.to_path_buf(),
            children,
            depth,
            index: config.index_name.clone(),
        })
    }
}

impl Display for DocFolder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let index = self.path.join(&self.index);
        // If this is the src folder
        if self.depth == 0 {
            writeln!(f, "[{}]({})", self.title, index.to_slash_lossy())?;
        } else {
            writeln!(
                f,
                "{}- [{}]({})",
                " ".repeat(((self.depth - 1) as usize) * 2),
                self.title,
                index.to_slash_lossy()
            )?;
        }

        // Files first
        for child in self
            .children
            .iter()
            .filter(|c| matches!(c, DocStructure::File(_)))
        {
            write!(f, "{}", child)?;
        }

        for child in self
            .children
            .iter()
            .filter(|c| matches!(c, DocStructure::Folder(_)))
        {
            write!(f, "{}", child)?;
        }

        Ok(())
    }
}

/// Represents a .md file inside the mdbook src folder
#[derive(Debug, Clone)]
pub struct DocFile {
    /// The title of the file, extracted from the first h1 heading
    /// Falls back to the filename if the file has no h1 heading
    pub title: String,
    /// The path to the file
    pub path: PathBuf,
    /// The depth of the file relative to the src folder
    pub depth: u16,
}

impl DocFile {
    pub fn new(title: impl Into<String>, path: PathBuf, depth: u16) -> Self {
        DocFile {
            title: title.into(),
            path,
            depth,
        }
    }

    /// Create a new DocFile from a path
    ///
    /// - If the file has an h1 heading, that will be used as the title.
    /// - If the file doesn't exist at the given path, this function returns None.
    /// - If the file doesn't have a '.md' extension, this function returns None.
    pub fn from_path(path: &Path, config: &Config, depth: u16) -> Option<Self> {
        if !path.is_file() {
            return None;
        }

        let filename = path.file_name().and_then(|p| p.to_str())?;

        if !filename.ends_with(".md") {
            return None;
        }

        if config.ignore_hidden && filename.starts_with(|c| c == '.' || c == '_') {
            return None;
        }

        let Ok(file) = File::open(path) else {
            warn!("Failed to open file: '{}'", path.display());
            return None;
        };

        let reader = BufReader::new(file);

        let lines = reader.lines().map_while(Result::ok);

        // Try to find an h1 heading
        for line in lines {
            if line.starts_with("# ") {
                let (_, mut title) = line
                    .split_once("# ")
                    .expect("Chapter title heading not found.");
                if let Some((title_without_comment, _)) = title.split_once("<!--") {
                    title = title_without_comment.trim();
                }
                return Some(Self::new(title, path.to_path_buf(), depth));
            }
        }
        // Fall back to filename
        let title = filename.to_string();

        Some(Self::new(title, path.to_path_buf(), depth))
    }
}

impl Display for DocFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // If this file is in the src root
        if self.depth == 0 || self.depth == 1 {
            return writeln!(f, "[{}]({})", self.title, self.path.to_slash_lossy());
        }

        writeln!(
            f,
            "{}- [{}]({})",
            " ".repeat(((self.depth - 1) as usize) * 2),
            self.title,
            self.path.to_slash_lossy()
        )
    }
}
