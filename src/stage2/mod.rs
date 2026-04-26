//! Stage-2 pipeline machinery. Houses the unified `Stage2Pipeline`
//! type (introduced in Phase 3), the `Wrap` trait that drives chain-N
//! dispatch (Phase 2), and the `Stage2Base` trait that connects
//! Stage-1 bases to their `Wrap` (Phase 2).

pub mod wrap;
pub mod base;

pub use wrap::{Wrap, Identity, SeedWrap};
pub use base::Stage2Base;
