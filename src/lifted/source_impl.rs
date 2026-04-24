//! LiftedPipeline source impls.
//!
//! `TreeishSource` impl (always, when Base is TreeishSource): the
//! lift chain transforms `(treeish, fold)` in two slots and the
//! continuation receives the result.
//!
//! `SeedSource` impl (when Base is SeedSource): the base's `grow`
//! is transported *covariantly* through the lift chain by
//! post-composition with `L::project_entry_node`. Any N-changing
//! lift is composable on the Seed path — the `N2 = Base::N`
//! constraint that the old 2-slot `Lift` trait needed is gone,
//! replaced by the honest functorial action of `Grow<Seed, ->`.

use std::marker::PhantomData;

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
      <Base as TreeishSource>::Domain: Domain<L::N2>,
      L: Lift<<Base as TreeishSource>::Domain,
              <Base as TreeishSource>::N,
              <Base as TreeishSource>::H,
              <Base as TreeishSource>::R>
         + Clone + Send + Sync + 'static,
      L::N2:   Clone + Send + Sync + 'static,
      L::MapH: Clone + 'static,
      L::MapR: Clone + 'static,
      <Base as SeedSource>::Seed: Send + Sync + 'static,
      <Base as TreeishSource>::N: Send + Sync + 'static,
      // The base's Grow handle must survive the 'static transport
      // closure. In the Shared domain it's Arc<dyn Fn + Send + Sync>,
      // already Send+Sync; the bound just surfaces that fact.
      <<Base as TreeishSource>::Domain as Domain<<Base as TreeishSource>::N>>::Grow<
          <Base as SeedSource>::Seed,
          <Base as TreeishSource>::N,
      >: Send + Sync + 'static,
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
            // Covariant transport: Grow<Seed, Base::N> → Grow<Seed, L::N2>.
            //   grow_transported(seed) = lift.project_entry_node(grow(seed))
            //
            // We clone the lift into a 'static closure (per PoC-A); all
            // shipped lifts are either Copy or Arc-backed so the clone
            // is O(1). The closure wraps into the domain's native Grow
            // handle via make_grow, preserving Send+Sync on Shared.
            let lift = self.pre_lift.clone();
            let grow_clone = grow;
            let grow_transported = <<Base as TreeishSource>::Domain as Domain<L::N2>>::make_grow::<
                <Base as SeedSource>::Seed, L::N2,
            >(move |seed: &<Base as SeedSource>::Seed| {
                let n = <<Base as TreeishSource>::Domain as Domain<
                    <Base as TreeishSource>::N,
                >>::invoke_grow::<<Base as SeedSource>::Seed, <Base as TreeishSource>::N>(
                    &grow_clone, seed,
                );
                lift.project_entry_node(n)
            });
            self.pre_lift.apply(treeish, fold, |treeish_out, fold_out| {
                cont(grow_transported, treeish_out, fold_out)
            })
        })
    }
}

// PhantomData sentinel — compile-only marker to confirm the ordering
// guarantee: `LiftedPipeline::SeedSource` requires `L: Send + Sync + 'static`
// for the grow-transport closure to typecheck as the domain's
// Grow<Seed, L::N2> handle.
#[allow(dead_code)]
fn _marker<Base, L>() -> PhantomData<(Base, L)> { PhantomData }
