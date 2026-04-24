//! `LiftedPipeline::TreeishSource` — the seedless TreeishPipeline-
//! rooted chain. The lift chain transforms `(treeish, fold)` and the
//! continuation receives the result.
//!
//! Under Option B the `SeedSource` impl was removed. The seed path
//! lives in `LiftedSeedPipeline` (see `lifted_seed/`).

use hylic::domain::Domain;
use hylic::ops::Lift;
use super::LiftedPipeline;
use super::super::source::TreeishSource;

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
