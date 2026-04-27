//! Blanket sugar traits — per-stage, per-domain.
//!
//! Stage-1 (on `SeedPipeline` / `TreeishPipeline`): reshape-based
//! sugars that stay at Stage 1.
//!
//! Stage-2 (on `Stage2Pipeline` via the deprecated `LiftedPipeline`
//! alias): chainable lift composition sugars. Phase 4 will unify
//! these with the seed-rooted inherent catalogue in
//! `crate::lifted_seed::sugars_*` via a `Wrap`-dispatched trait.
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
pub use lifted_shared::Stage2SugarsShared;
pub use lifted_local::Stage2SugarsLocal;
// Back-compat re-export (Phase 11 retires it).
#[allow(deprecated)]
#[deprecated(note = "renamed to Stage2SugarsShared")]
pub use lifted_shared::Stage2SugarsShared as LiftedSugarsShared;
#[allow(deprecated)]
#[deprecated(note = "renamed to Stage2SugarsLocal")]
pub use lifted_local::Stage2SugarsLocal as LiftedSugarsLocal;
