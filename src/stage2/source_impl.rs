//! `Stage2Pipeline::TreeishSource` — for chains whose lift L is
//! typed at `Base::N` (= treeish-rooted chains, where the chain's
//! input N is plain N).
//!
//! Seed-rooted chains where L is typed at `SeedNode<N>` do NOT match
//! this bound and so do not impl `TreeishSource` via this path —
//! they expose `.run` directly via inherent impls in
//! `crate::stage2::run_seed_*.rs`.

use hylic::domain::Domain;
use hylic::ops::Lift;
use crate::source::TreeishSource;
use super::pipeline::Stage2Pipeline;

impl<Base, L> TreeishSource for Stage2Pipeline<Base, L>
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
