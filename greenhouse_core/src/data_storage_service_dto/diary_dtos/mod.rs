//! Diary entry data transfer objects.
//!
//! The diary is a freeform log where users can record manual observations about
//! the greenhouse — planting notes, treatment records, harvest logs, etc. Entries
//! can be tagged for easier filtering and search.
//!
//! # Modules
//!
//! - [`endpoints`] — REST path constants
//! - [`get_diary`] — response type for listing all diary entries
//! - [`get_diary_entry`] — response type for a single diary entry
//! - [`post_diary_entry`] — request type for creating a new entry
//! - [`put_diary_entry`] — request type for updating an existing entry
//! - [`post_diary_tag`] — request type for adding a tag to an entry

pub mod endpoints;
pub mod get_diary;
pub mod get_diary_entry;
pub mod post_diary_entry;
pub mod post_diary_tag;
pub mod put_diary_entry;
