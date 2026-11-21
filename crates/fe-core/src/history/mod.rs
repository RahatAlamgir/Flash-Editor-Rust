//! Undo / redo. One history per project, shared by all its pages (like the C# `ProjectHistory`).
//!
//! Every step keeps a copy of the page's document from before and after the change. Copies are cheap: layers
//! share their tiles, so a step only costs memory for the tiles that really changed. Rules from the C#
//! version: 30 steps by default (1 to 60), about 1 GB at most (the oldest steps are forgotten first), small
//! repeated changes such as dragging the opacity slider merge into one step.

mod size;
mod undo_stack;

pub use undo_stack::{DEFAULT_STEPS, History, MAX_BYTES, MAX_STEPS, MIN_STEPS, MergeKey, Step};
