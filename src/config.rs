use std::convert::Infallible;

use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct Config {
    pub index_name: String,
    pub ignore_hidden: bool,
}

impl Config {
    pub fn new(index_name: String, ignore_hidden: bool) -> Self {
        Config {
            index_name,
            ignore_hidden,
        }
    }

    /// Try to load the config from book.toml or return the default config
    pub fn from_mdbook(book_conf: &mdbook::Config) -> Self {
        match book_conf.get("preprocessor.autosummary") {
            Some(raw) => raw
                .clone()
                .try_into()
                .or_else(|_| Ok::<Self, Infallible>(Config::default()))
                .unwrap(),
            None => Config::default(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            index_name: "index.md".to_string(),
            ignore_hidden: true,
        }
    }
}
