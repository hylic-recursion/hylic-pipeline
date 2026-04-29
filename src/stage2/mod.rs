//! Stage-2 pipeline machinery — unified across treeish-rooted and
//! seed-rooted bases.
//!
//! - [`Stage2Pipeline<Base, L>`] is the unified Stage-2 typestate.
//! - [`Wrap`] is the type-level dispatch trait for chain N wrapping.
//! - [`wrap::WrapShared`] / [`wrap::WrapLocal`] add per-domain build
//!   methods used by the unified Stage-2 sugar surface.
//! - [`Stage2Base`] connects Stage-1 bases to their `Wrap`.

pub mod pipeline;
pub mod primitives;
pub mod base;
pub mod source;
pub mod wrap;
pub mod run;

pub use pipeline::Stage2Pipeline;
pub use base::{Stage2Base, Stage2BaseSlice};
pub use wrap::{Wrap, Identity, SeedWrap};
pub use wrap::shared::WrapShared;
pub use wrap::local::WrapLocal;
