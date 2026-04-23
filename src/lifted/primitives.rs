//! Stage-2 primitives on `LiftedPipeline`.
//!
//!   - `then_lift(outer)` — post-compose a new Lift onto the chain.
//!     Domain-generic; `outer`'s input types must match the current
//!     tip's output types. Sole composition primitive.
//!   - `before_lift(first)` / `before_lift_local(first)` —
//!     pre-compose a **type-preserving** Lift before the chain.
//!     Domain-specific because `L0`'s input/output types must both
//!     equal the base's input types (the chain's existing input
//!     types are monomorphic). Use `n_lift`/`map_r_bi_lift`/
//!     `phases_lift` for variance-aware pre-adaptation.
//!
//! User-facing sugars (wrap_init, map_r_bi, filter_edges, …) live
//! on the `LiftedSugarsShared` / `LiftedSugarsLocal` blanket traits
//! in `sugars/`.

use hylic::domain::{Domain, Local, Shared};
use hylic::ops::{ComposedLift, Lift};
use super::LiftedPipeline;
use super::super::source::TreeishSource;

// ── then_lift — domain-generic post-compose ────────────────────

impl<Base, L> LiftedPipeline<Base, L>
where Base: TreeishSource,
      <Base as TreeishSource>::Domain: Domain<L::N2>,
      L: Lift<<Base as TreeishSource>::Domain,
              <Base as TreeishSource>::N,
              <Base as TreeishSource>::H,
              <Base as TreeishSource>::R>,
{
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
}

// ── before_lift — Shared-domain pre-compose ────────────────────

impl<Base, L> LiftedPipeline<Base, L>
where
    Base: TreeishSource<Domain = Shared>,
    Shared: Domain<L::N2>,
    L: Lift<Shared,
            <Base as TreeishSource>::N,
            <Base as TreeishSource>::H,
            <Base as TreeishSource>::R>,
    L::N2:   Clone + 'static,
    L::MapH: Clone + 'static,
    L::MapR: Clone + 'static,
{
    pub fn before_lift<L0>(self, first: L0) -> LiftedPipeline<Base, ComposedLift<L0, L>>
    where L0: Lift<Shared, Base::N, Base::H, Base::R>,
          Shared: Domain<L0::N2>,
    {
        LiftedPipeline { base: self.base, pre_lift: ComposedLift::compose(first, self.pre_lift) }
    }
}

// ── before_lift_local — Local-domain pre-compose ───────────────

impl<Base, L> LiftedPipeline<Base, L>
where
    Base: TreeishSource<Domain = Local>,
    Local: Domain<L::N2>,
    L: Lift<Local,
            <Base as TreeishSource>::N,
            <Base as TreeishSource>::H,
            <Base as TreeishSource>::R>,
    L::N2:   Clone + 'static,
    L::MapH: Clone + 'static,
    L::MapR: Clone + 'static,
{
    pub fn before_lift_local<L0>(self, first: L0) -> LiftedPipeline<Base, ComposedLift<L0, L>>
    where L0: Lift<Local, Base::N, Base::H, Base::R>,
          Local: Domain<L0::N2>,
    {
        LiftedPipeline { base: self.base, pre_lift: ComposedLift::compose(first, self.pre_lift) }
    }
}
