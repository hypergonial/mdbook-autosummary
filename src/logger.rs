use std::env;

use chrono::Local;
use env_logger::Builder;
use log::LevelFilter;
use std::io::Write;

/// Source: https://github.com/rust-lang/mdBook/blob/master/src/main.rs#L97
/// Sets up the logger in a way that matches mdbook's implementation
pub(crate) fn init_logger() {
    let mut builder = Builder::new();

    builder.format(|formatter, record| {
        writeln!(
            formatter,
            "{} [{}] ({}): {}",
            Local::now().format("%Y-%m-%d %H:%M:%S"),
            record.level(),
            record.target(),
            record.args()
        )
    });

    if let Ok(var) = env::var("RUST_LOG") {
        builder.parse_filters(&var);
    } else {
        // if no RUST_LOG provided, default to logging at the Info level
        builder.filter(None, LevelFilter::Info);
        // Filter extraneous html5ever not-implemented messages
        builder.filter(Some("html5ever"), LevelFilter::Error);
    }

    builder.init();
}
