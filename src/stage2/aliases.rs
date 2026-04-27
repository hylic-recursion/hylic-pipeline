//! Deprecated type aliases — `LiftedPipeline` and `LiftedSeedPipeline`.
//!
//! Both collapsed into the single [`Stage2Pipeline`] in the
//! seed-pipeline-unification cycle. Aliases retained for one cycle
//! to ease external migration; Phase 11 retires them.
//!
//! [`Stage2Pipeline`]: super::Stage2Pipeline

use hylic::ops::IdentityLift;
use super::Stage2Pipeline;

/// Deprecated alias for [`Stage2Pipeline`]. The two historical
/// Stage-2 types collapsed into one form distinguished only by which
/// `Base` they wrap.
#[deprecated(note = "use Stage2Pipeline")]
#[allow(type_alias_bounds)]
pub type LiftedPipeline<Base, L = IdentityLift> = Stage2Pipeline<Base, L>;

/// Deprecated alias for [`Stage2Pipeline`].
#[deprecated(note = "use Stage2Pipeline")]
#[allow(type_alias_bounds)]
pub type LiftedSeedPipeline<Base, L = IdentityLift> = Stage2Pipeline<Base, L>;
