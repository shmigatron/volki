//! Filesystem operations — file I/O, directories, metadata.

pub mod dir;
pub mod file;
pub mod metadata;

pub use dir::{create_dir, create_dir_all, read_dir, remove_dir, remove_dir_all, remove_file, DirEntry, FileType, ReadDir};
pub use file::{read, read_to_string, write, write_str, File};
pub use metadata::{exists, is_dir, is_file, metadata, Metadata};
