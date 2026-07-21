//! File I/O operations with codec support
//!
//! This module provides convenient file I/O operations that integrate with
//! the codec abstraction. All operations support automatic directory creation
//! and atomic writes (write to temporary file, then rename).
//!
//! # Examples
//!
//! ```no_run
//! use z00z_utils::io::{save_json, load_json};
//! use serde::{Serialize, Deserialize};
//!
//! #[derive(Serialize, Deserialize)]
//! struct Config {
//!     name: String,
//!     port: u16,
//! }
//!
//! let config = Config { name: "app".into(), port: 8080 };
//! save_json("config.json", &config)?;
//!
//! let loaded: Config = load_json("config.json")?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

pub mod error;
pub mod fs;
mod spool;

pub use std::fs::File;
pub use std::io::copy;
pub use std::io::{Cursor, Read, Seek, Write};

pub use error::IoError;
pub use fs::{
    atomic_write_file_private, atomic_write_file_streaming, create_dir_all, current_exe_run_root,
    file_len, hash_root_inputs, load_bincode, load_bincode_bounded, load_json, load_json_bounded,
    load_with_codec, load_yaml, load_yaml_bounded, open_lock_file, path_exists,
    path_exists_no_follow, prepare_managed_root, prune_hex_dirs, prune_scope_alias_dirs, read_dir,
    read_file, read_file_bounded, read_link, read_to_string, remove_dir_all, remove_file,
    rename_file, reset_managed_root, reset_managed_root_once, save_bincode, save_json,
    save_with_codec, save_yaml, set_file_mode, set_permissions_mode, stable_current_exe_scope,
    symlink_metadata, sync_directory, write_file, write_file_private_new,
};
pub use spool::PrivateSpoolFile;
