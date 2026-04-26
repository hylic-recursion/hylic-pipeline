//! `LiftedPipeline` — deprecated alias for `Stage2Pipeline`.
//!
//! The two Stage-2 types (`LiftedPipeline` and `LiftedSeedPipeline`)
//! collapsed into a single `Stage2Pipeline<Base, L>` distinguished
//! only by which Base they wrap. This alias is kept for one cycle.

use hylic::ops::IdentityLift;

#[deprecated(note = "use Stage2Pipeline (single Stage-2 type for both treeish-rooted and seed-rooted)")]
#[allow(type_alias_bounds)]
pub type LiftedPipeline<Base, L = IdentityLift> = crate::stage2::Stage2Pipeline<Base, L>;
