//! Knowledge source fetchers (v0.0.422).
//!
//! Fetches content from:
//! - Man pages (local)
//! - Help output (--help, -h)
//! - Arch Wiki (cache or online)
//! - Local documentation
//! - Pacman package info

mod help;
mod local_doc;
mod man_page;
mod pacman;
mod types;
mod utils;
mod wiki;

// Re-export public types and functions
pub use help::fetch_help_output;
pub use local_doc::fetch_local_doc;
pub use man_page::fetch_man_page;
pub use pacman::fetch_pacman_info;
pub use types::SourceFetchResult;
pub use wiki::fetch_arch_wiki;
