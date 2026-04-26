//! `LiftedSeedPipeline` — deprecated alias for `Stage2Pipeline`.
//!
//! The two Stage-2 types collapsed into a single `Stage2Pipeline<Base, L>`.
//! All run methods live in `crate::stage2::run_seed_*`. Sugar
//! catalogues (`sugars_shared`/`sugars_local` here) target the alias
//! and so apply to `Stage2Pipeline<SeedPipeline<...>, L>` directly.
//!
//! Phase 4 of the seed-pipeline-unification plan will fold these
//! per-Base inherent sugars into a Wrap-dispatched unified surface.
//! For now they remain.

use hylic::ops::IdentityLift;

pub mod sugars_shared;
pub mod sugars_local;
pub(crate) mod gat_helpers;

/// Deprecated alias for [`crate::stage2::Stage2Pipeline`]. The two
/// historical Stage-2 types collapsed into a single
/// `Stage2Pipeline<Base, L>`. Kept for one cycle to ease external
/// migration; new code should refer to `Stage2Pipeline` directly.
#[deprecated(note = "use Stage2Pipeline (single Stage-2 type for both treeish-rooted and seed-rooted)")]
#[allow(type_alias_bounds)]
pub type LiftedSeedPipeline<Base, L = IdentityLift> = crate::stage2::Stage2Pipeline<Base, L>;
