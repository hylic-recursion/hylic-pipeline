//! SeedPipeline's `TreeishSource` impl — fuses grow + seeds_from_node
//! into a plain `Graph<N>` at yield time so the seed structure can be
//! traversed by a `run_from_node` executor.
//!
//! The seed axis as a *lift-chain* input is handled by
//! `LiftedSeedPipeline` (see `lifted_seed/`). `SeedSource` and
//! `with_seeded` were removed by the Option-B pivot.

use hylic::domain::Domain;
use hylic::ops::{IdentityLift, SeedNode, ShapeCapable};
use super::SeedPipeline;
use super::super::stage2::Stage2Pipeline;
use super::super::source::TreeishSource;

impl<D, N, Seed, H, R> TreeishSource for SeedPipeline<D, N, Seed, H, R>
where D: ShapeCapable<N>,
      N: Clone + 'static, Seed: Clone + 'static,
      H: Clone + 'static, R: Clone + 'static,
      <D as Domain<N>>::Grow<Seed, N>: Clone,
      <D as Domain<N>>::Graph<Seed>:   Clone,
      <D as Domain<N>>::Fold<H, R>:    Clone,
{
    type Domain = D;
    type N = N;
    type H = H;
    type R = R;

    fn with_treeish<T>(
        &self,
        cont: impl FnOnce(
            <D as Domain<N>>::Graph<N>,
            <D as Domain<N>>::Fold<H, R>,
        ) -> T,
    ) -> T {
        let treeish = D::fuse_grow_with_seeds::<Seed>(
            self.grow.clone(),
            self.seeds_from_node.clone(),
        );
        cont(treeish, self.fold.clone())
    }
}

// ── Transition to Stage 2 ──────────────────────────────

impl<D, N, Seed, H, R> SeedPipeline<D, N, Seed, H, R>
where D: Domain<N> + Domain<SeedNode<N>>,
      N: 'static, Seed: 'static, H: 'static, R: 'static,
{
    /// Transition to Stage 2. Produces a `Stage2Pipeline` whose
    /// chain is typed at `SeedNode<N>`. SeedLift is NOT yet
    /// constructed — it's assembled at `.run` time from user-supplied
    /// `root_seeds` and `entry_heap`.
    pub fn lift(self) -> Stage2Pipeline<Self, IdentityLift> {
        Stage2Pipeline::new(self, IdentityLift)
    }
}
