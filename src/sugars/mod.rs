//! Blanket sugar traits — per-stage, per-domain.
//!
//! Stage-1 (on `SeedPipeline` / `TreeishPipeline`): reshape-based
//! sugars that stay at Stage 1.
//!
//! Stage-2 (on `LiftedPipeline` — and on Stage-1 pipelines via
//! auto-lift): chainable lift composition sugars.
//!
//! Each sugar is a trait method; the trait has one impl per
//! (pipeline-type × domain). The trait is in scope via
//! `hylic_pipeline::prelude::*`.

pub mod seed_shared;
pub mod seed_local;
pub mod treeish_shared;
pub mod treeish_local;
pub mod lifted_shared;
pub mod lifted_local;

pub use seed_shared::SeedSugarsShared;
pub use seed_local::SeedSugarsLocal;
pub use treeish_shared::TreeishSugarsShared;
pub use treeish_local::TreeishSugarsLocal;
pub use lifted_shared::LiftedSugarsShared;
pub use lifted_local::LiftedSugarsLocal;
