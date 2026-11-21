//! What the menus, buttons and shortcuts do. One file per area: edit (undo / redo), file (open, save,
//! close...), layers, pages, selection (copy / paste, select all, split, reveal all...), checks (may this
//! layer be changed?). More areas join in later phases.

pub mod adjustments;
pub mod checks;
pub mod codes;
pub mod edit;
pub mod effects;
pub mod file;
pub mod frames;
pub mod image;
pub mod insert;
pub mod layers;
pub mod mask;
pub mod objects;
pub mod pages;
pub mod parts;
pub mod selection;
pub mod styles;
pub mod text;
pub mod tutorial;
