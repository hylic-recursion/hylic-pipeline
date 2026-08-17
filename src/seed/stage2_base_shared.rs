//! `Stage2Base` + `Stage2BaseSlice` impls for `SeedPipeline<Shared, …>`.
//!
//! Per-domain (the GAT crossing for `<Shared as Domain<N>>::Grow<Seed, N>`
//! does not normalise inside a generic-impl body, so the `SeedLift`
//! constructor call has to happen here, where `Self = Shared` is fixed
//! and the GAT helpers can pin the conversion).
//!
//! `RunInputs<'i, CurN>` is the owned pair
//! `(<Shared as Domain<()>>::Graph<Seed>, H)`; the `'i` lifetime and
//! `CurN` parameter are unused at the value level — `EntryRoot` is
//! constructible at any `CurN`.

use std::sync::Arc;

use hylic::domain::{Domain, Shared};
use hylic::ops::seed_node_internal as sn_int;
use hylic::ops::{SeedLift, ShapeCapable};

use super::SeedPipeline;
use crate::stage2::run::gat_helpers::shared_grow_as_arc;
use crate::stage2::{Stage2Base, Stage2BaseSlice, Wrap};

impl<N, Seed, H, R> Stage2Base for SeedPipeline<Shared, N, Seed, H, R>
where
    N: Clone + Send + Sync + 'static,
    Seed: Clone + Send + Sync + 'static,
    H: Clone + Send + Sync + 'static,
    R: Clone + Send + Sync + 'static,
    Shared: ShapeCapable<N>,
    <Shared as Domain<N>>::Grow<Seed, N>: Clone,
    <Shared as Domain<N>>::Graph<Seed>: Clone,
    <Shared as Domain<N>>::Fold<H, R>: Clone,
{
    type Wrap = crate::stage2::SeedWrap;
    type UserN = N;
    type RunInputs<'i, CurN: Clone + 'static> = (<Shared as Domain<()>>::Graph<Seed>, H);
    type PreLift = SeedLift<Shared, N, Seed, H>;

    fn provide_run_essentials<CurN: Clone + 'static, T>(
        &self,
        inputs: Self::RunInputs<'_, CurN>,
        cont: impl FnOnce(Self::PreLift, &<Self::Wrap as Wrap>::Of<CurN>) -> T,
    ) -> T {
        let (root_seeds, entry_heap) = inputs;
        let grow_arc: Arc<dyn Fn(&Seed) -> N + Send + Sync> = shared_grow_as_arc::<Seed, N>(self.grow.clone());
        let pre = SeedLift::from_arc_grow(grow_arc, root_seeds, move || {
            entry_heap.clone()
        });
        // Synthetic root constructed at the post-chain type `CurN`,
        // materialised in this frame so the borrow outlives `cont`.
        let entry_root = sn_int::entry_root::<CurN>();
        cont(pre, &entry_root)
    }
}

impl<N, Seed, H, R> Stage2BaseSlice for SeedPipeline<Shared, N, Seed, H, R>
where
    N: Clone + Send + Sync + 'static,
    Seed: Clone + Send + Sync + 'static,
    H: Clone + Send + Sync + 'static,
    R: Clone + Send + Sync + 'static,
    Shared: ShapeCapable<N>,
    <Shared as Domain<N>>::Grow<Seed, N>: Clone,
    <Shared as Domain<N>>::Graph<Seed>: Clone,
    <Shared as Domain<N>>::Fold<H, R>: Clone,
{
    type Seed = Seed;

    fn slice_run_inputs<CurN: Clone + 'static>(seeds: &[Seed], heap: H) -> Self::RunInputs<'static, CurN> {
        let owned: Vec<Seed> = seeds.to_vec();
        let es: <Shared as Domain<()>>::Graph<Seed> = hylic::graph::edgy_visit(
            move |_: &(), cb: &mut dyn FnMut(&Seed)| {
                for s in &owned {
                    cb(s);
                }
            },
        );
        (es, heap)
    }
}
