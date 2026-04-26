//! Stage-2 primitives on `LiftedSeedPipeline`, domain-generic.
//!
//! The single load-bearing line is in the impl's where-clause:
//! `L: Lift<D, SeedNode<N>, H, R>`. Everything else is ordinary
//! plumbing for seedless Stage-2 composition.

use hylic::domain::Domain;
use hylic::ops::{ComposedLift, Lift, SeedNode};
use super::LiftedSeedPipeline;
use super::super::seed::SeedPipeline;

impl<D, N, Seed, H, R, L> LiftedSeedPipeline<SeedPipeline<D, N, Seed, H, R>, L>
where D: Domain<N> + Domain<SeedNode<N>> + Domain<L::N2>,
      N: Clone + 'static, Seed: Clone + 'static,
      H: Clone + 'static, R: Clone + 'static,
      L: Lift<D, SeedNode<N>, H, R>,
      L::N2:   Clone + 'static,
      L::MapH: Clone + 'static,
      L::MapR: Clone + 'static,
{
    /// Post-compose `outer` onto the chain. `outer`'s inputs must
    /// match the tip's outputs.
    pub fn then_lift<L2>(
        self,
        outer: L2,
    ) -> LiftedSeedPipeline<SeedPipeline<D, N, Seed, H, R>, ComposedLift<L, L2>>
    where D: Domain<L2::N2>,
          L2: Lift<D, L::N2, L::MapH, L::MapR>,
          L2::N2:   Clone + 'static,
          L2::MapH: Clone + 'static,
          L2::MapR: Clone + 'static,
    {
        LiftedSeedPipeline {
            base:     self.base,
            pre_lift: ComposedLift::compose(self.pre_lift, outer),
        }
    }
}
