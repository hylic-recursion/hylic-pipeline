//! Stage-2 primitives on `LiftedPipeline`, domain-generic.
//!
//! - `then_lift(outer)` — post-compose a Lift onto the chain. Sole
//!   composition primitive; `outer`'s inputs must match the tip's
//!   outputs.
//! - `before_lift(first)` — pre-compose a **type-preserving** Lift
//!   before the chain. `first`'s outputs must equal Base's inputs.
//!   Axis-selective pre-adaptation uses the variance-aware sugars
//!   (`map_node_bi`, `map_r_bi`, `n_lift`, `phases_lift`) instead.
//!
//! User-facing sugars (wrap_init, map_r_bi, filter_edges, …) live
//! on the `LiftedSugarsShared` / `LiftedSugarsLocal` blanket traits
//! in `sugars/`.

use hylic::domain::Domain;
use hylic::ops::{ComposedLift, Lift};
use super::LiftedPipeline;
use super::super::source::TreeishSource;

impl<Base, L> LiftedPipeline<Base, L>
where Base: TreeishSource,
      <Base as TreeishSource>::Domain: Domain<L::N2>,
      L: Lift<<Base as TreeishSource>::Domain,
              <Base as TreeishSource>::N,
              <Base as TreeishSource>::H,
              <Base as TreeishSource>::R>,
{
    /// Sole composition primitive: post-compose `outer` onto the chain.
    pub fn then_lift<L2>(
        self,
        outer: L2,
    ) -> LiftedPipeline<Base, ComposedLift<L, L2>>
    where <Base as TreeishSource>::Domain: Domain<L2::N2>,
          L2: Lift<<Base as TreeishSource>::Domain, L::N2, L::MapH, L::MapR>,
    {
        LiftedPipeline {
            base:     self.base,
            pre_lift: ComposedLift::compose(self.pre_lift, outer),
        }
    }

    /// Pre-compose a type-preserving lift `first` before the chain.
    /// Restricted to `L0` whose outputs equal Base's inputs (Rust
    /// enforces this via the use-site `ComposedLift<L0, L>` bound).
    pub fn before_lift<L0>(self, first: L0) -> LiftedPipeline<Base, ComposedLift<L0, L>>
    where L0: Lift<<Base as TreeishSource>::Domain, Base::N, Base::H, Base::R>,
          <Base as TreeishSource>::Domain: Domain<L0::N2>,
    {
        LiftedPipeline { base: self.base, pre_lift: ComposedLift::compose(first, self.pre_lift) }
    }
}
