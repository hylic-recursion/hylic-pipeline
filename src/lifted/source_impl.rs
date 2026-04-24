//! LiftedPipeline source impls.
//!
//! `TreeishSource` impl (always, when Base is TreeishSource): the
//! lift chain transforms `(treeish, fold)` in two slots and the
//! continuation receives the result.
//!
//! `SeedSource` impl (when Base is SeedSource): the base's grow is
//! threaded to the continuation *unchanged*. Because the 2-slot
//! `Lift` trait cannot transform grow, the SeedSource impl is
//! constrained to lift chains that preserve N (`L::N2 = Base::N`).
//! Pipelines that need Stage-2 N-change must reshape at Stage 1
//! via `map_node_bi` / `reshape`, or compose `SeedLift` explicitly
//! (which retires the Seed axis) and run through `run_from_node`.

use hylic::domain::Domain;
use hylic::ops::Lift;
use super::LiftedPipeline;
use super::super::source::{TreeishSource, SeedSource};

impl<Base, L> TreeishSource for LiftedPipeline<Base, L>
where Base: TreeishSource,
      <Base as TreeishSource>::Domain: Domain<L::N2>,
      L: Lift<<Base as TreeishSource>::Domain,
              <Base as TreeishSource>::N,
              <Base as TreeishSource>::H,
              <Base as TreeishSource>::R>,
      L::N2:   Clone + 'static,
      L::MapH: Clone + 'static,
      L::MapR: Clone + 'static,
{
    type Domain = <Base as TreeishSource>::Domain;
    type N = L::N2;
    type H = L::MapH;
    type R = L::MapR;

    fn with_treeish<T>(
        &self,
        cont: impl FnOnce(
            <Self::Domain as Domain<Self::N>>::Graph<Self::N>,
            <Self::Domain as Domain<Self::N>>::Fold<Self::H, Self::R>,
        ) -> T,
    ) -> T {
        self.base.with_treeish(|treeish, fold| {
            self.pre_lift.apply(treeish, fold,
                |treeish_out, fold_out| cont(treeish_out, fold_out),
            )
        })
    }
}

impl<Base, L> SeedSource for LiftedPipeline<Base, L>
where Base: SeedSource,
      // The lift chain must preserve N so the base's grow type
      // stays compatible with the yielded triple. Stage-2
      // N-change lifts cannot feed a SeedSource path.
      L: Lift<<Base as TreeishSource>::Domain,
              <Base as TreeishSource>::N,
              <Base as TreeishSource>::H,
              <Base as TreeishSource>::R,
              N2 = <Base as TreeishSource>::N>,
      L::MapH: Clone + 'static,
      L::MapR: Clone + 'static,
{
    type Seed = <Base as SeedSource>::Seed;

    fn with_seeded<T>(
        &self,
        cont: impl FnOnce(
            <Self::Domain as Domain<Self::N>>::Grow<Self::Seed, Self::N>,
            <Self::Domain as Domain<Self::N>>::Graph<Self::N>,
            <Self::Domain as Domain<Self::N>>::Fold<Self::H, Self::R>,
        ) -> T,
    ) -> T {
        self.base.with_seeded(|grow, treeish, fold| {
            self.pre_lift.apply(treeish, fold, |treeish_out, fold_out| {
                cont(grow, treeish_out, fold_out)
            })
        })
    }
}
