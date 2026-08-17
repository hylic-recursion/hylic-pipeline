//! Blanket sugar traits — per-stage, per-domain.
//!
//! Stage-1 (on `SeedPipeline` / `TreeishPipeline`): reshape-based
//! sugars that stay at Stage 1 (`SeedSugars*`, `TreeishSugars*`).
//!
//! Stage-2 (on `Stage2Pipeline` over either Base): chainable
//! lift-composition sugars dispatched through `Wrap`
//! (`Stage2SugarsShared`, `Stage2SugarsLocal`). One canonical body
//! per sugar; closures over `&UN` are peeled on `SeedWrap`-rooted
//! chains and pass-through on `Identity`-rooted chains.
//!
//! All sugar traits come into scope via `hylic_pipeline::prelude::*`.

pub mod seed_local;
pub mod seed_shared;
pub mod stage2_local;
pub mod stage2_shared;
pub mod treeish_local;
pub mod treeish_shared;

pub use seed_local::SeedSugarsLocal;
pub use seed_shared::SeedSugarsShared;
pub use stage2_local::Stage2SugarsLocal;
pub use stage2_shared::Stage2SugarsShared;
pub use treeish_local::TreeishSugarsLocal;
pub use treeish_shared::TreeishSugarsShared;
