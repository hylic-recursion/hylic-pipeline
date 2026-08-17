//! Stage-2 pipeline machinery — unified across treeish-rooted and
//! seed-rooted bases.
//!
//! - [`Stage2Pipeline<Base, L>`] is the unified Stage-2 typestate.
//! - [`Wrap`] is the type-level dispatch trait for chain N wrapping.
//! - [`WrapShared`] / [`WrapLocal`] add per-domain build methods
//!   used by the unified Stage-2 sugar surface.
//! - [`Stage2Base`] connects Stage-1 bases to their `Wrap`.

pub mod base;
pub mod pipeline;
pub mod primitives;
pub mod run;
pub mod source;
pub mod wrap;

pub use base::{Stage2Base, Stage2BaseSlice};
pub use pipeline::Stage2Pipeline;
pub use wrap::local::WrapLocal;
pub use wrap::shared::WrapShared;
pub use wrap::{Identity, SeedWrap, Wrap};
