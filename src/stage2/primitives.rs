//! Stage-2 primitives on `Stage2Pipeline<Base, L>`.
//!
//! `then_lift` is the sole composition primitive — it is unconstrained
//! at construction (just struct manipulation). The validity of any
//! particular composition is enforced where it is *consumed*: the
//! `Lift` impl on `ComposedLift<L1, L2>` demands the obvious join,
//! the `TreeishSource` impl on `Stage2Pipeline` demands `L: Lift<...>`,
//! and the `.run_*` inherent impls demand the same. A pipeline holding
//! a nonsensical chain is structurally typeable; it just cannot `.run`.
//!
//! This shape resolves the dispatch ambiguity that arose when the two
//! Stage-2 structs collapsed: a single unconstrained inherent on
//! `Stage2Pipeline<Base, L>` is unambiguously selectable from any
//! caller (trait body or otherwise), regardless of how Base is
//! constrained at the call site.
//!
//! `before_lift` is treeish-rooted only — pre-composing before
//! `SeedLift` (which is the natural seed-rooted chain head) is not a
//! sensible position. It carries the `L0` validity bound directly.

use hylic::domain::Domain;
use hylic::ops::{ComposedLift, Lift};
use crate::treeish::TreeishPipeline;
use super::pipeline::Stage2Pipeline;

// ── Sole composition primitive: unconstrained ────────────

impl<Base, L> Stage2Pipeline<Base, L> {
    // ANCHOR: then_lift_primitive
    /// Post-compose `outer` onto the chain. Pure struct construction;
    /// no bounds. The composition's *meaningfulness* is enforced where
    /// the chain is consumed (`.run_*`, `TreeishSource`).
    pub fn then_lift<L2>(
        self,
        outer: L2,
    ) -> Stage2Pipeline<Base, ComposedLift<L, L2>> {
        Stage2Pipeline {
            base:     self.base,
            pre_lift: ComposedLift::compose(self.pre_lift, outer),
        }
    }
    // ANCHOR_END: then_lift_primitive
}

// ── Pre-composition: treeish-rooted only ─────────────────

impl<D, N, H, R, L> Stage2Pipeline<TreeishPipeline<D, N, H, R>, L>
where D: Domain<N>,
      N: Clone + 'static, H: Clone + 'static, R: Clone + 'static,
      <D as Domain<N>>::Graph<N>:   Clone,
      <D as Domain<N>>::Fold<H, R>: Clone,
      L: Lift<D, N, H, R>,
      D: Domain<L::N2>,
{
    // ANCHOR: before_lift_primitive
    /// Pre-compose a type-preserving lift `first` before the chain.
    /// `first`'s output (N, H, R) must equal the base's input.
    /// For non-type-preserving pre-adaptation, use the variance-aware
    /// sugars (`map_node_bi`, `map_r_bi`, `n_lift`, `phases_lift`).
    ///
    /// Available only for treeish-rooted pipelines: seed-rooted
    /// chains have `SeedLift` composed at `.run` time as the natural
    /// chain head, leaving no meaningful "before" position.
    pub fn before_lift<L0>(self, first: L0)
        -> Stage2Pipeline<TreeishPipeline<D, N, H, R>, ComposedLift<L0, L>>
    where L0: Lift<D, N, H, R>,
          D: Domain<L0::N2>,
    {
        Stage2Pipeline { base: self.base, pre_lift: ComposedLift::compose(first, self.pre_lift) }
    }
    // ANCHOR_END: before_lift_primitive
}
