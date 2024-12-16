//! Derive macro input arguments.
mod entry;
mod model;

mod ty;

#[cfg(any(feature = "uuid"))]
mod external;
#[cfg(feature = "uuid")]
mod uuid;

pub(super) use entry::{EntryArgs, EntryPosition};
pub(super) use model::{ModelArgs, ModelTableType};
