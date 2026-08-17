//! `Stage2Base` + `Stage2BaseSlice` impls for `SeedPipeline<Local, …>`.
//!
//! Mirror of the Shared impl: `Rc`-storage, no `Send + Sync` bounds.

use std::rc::Rc;

use hylic::domain::local::edgy as local_edgy;
use hylic::domain::{Domain, Local};
use hylic::ops::seed_node_internal as sn_int;
use hylic::ops::{SeedLift, ShapeCapable};

use super::SeedPipeline;
use crate::stage2::run::gat_helpers::local_grow_as_rc;
use crate::stage2::{Stage2Base, Stage2BaseSlice, Wrap};

impl<N, Seed, H, R> Stage2Base for SeedPipeline<Local, N, Seed, H, R>
where
    N: Clone + 'static,
    Seed: Clone + 'static,
    H: Clone + 'static,
    R: Clone + 'static,
    Local: ShapeCapable<N>,
    <Local as Domain<N>>::Grow<Seed, N>: Clone,
    <Local as Domain<N>>::Graph<Seed>: Clone,
    <Local as Domain<N>>::Fold<H, R>: Clone,
{
    type Wrap = crate::stage2::SeedWrap;
    type UserN = N;
    type RunInputs<'i, CurN: Clone + 'static> = (<Local as Domain<()>>::Graph<Seed>, H);
    type PreLift = SeedLift<Local, N, Seed, H>;

    fn provide_run_essentials<CurN: Clone + 'static, T>(
        &self,
        inputs: Self::RunInputs<'_, CurN>,
        cont: impl FnOnce(Self::PreLift, &<Self::Wrap as Wrap>::Of<CurN>) -> T,
    ) -> T {
        let (root_seeds, entry_heap) = inputs;
        let grow_rc: Rc<dyn Fn(&Seed) -> N> = local_grow_as_rc::<Seed, N>(self.grow.clone());
        let pre = SeedLift::from_rc_grow(grow_rc, root_seeds, move || {
            entry_heap.clone()
        });
        let entry_root = sn_int::entry_root::<CurN>();
        cont(pre, &entry_root)
    }
}

impl<N, Seed, H, R> Stage2BaseSlice for SeedPipeline<Local, N, Seed, H, R>
where
    N: Clone + 'static,
    Seed: Clone + 'static,
    H: Clone + 'static,
    R: Clone + 'static,
    Local: ShapeCapable<N>,
    <Local as Domain<N>>::Grow<Seed, N>: Clone,
    <Local as Domain<N>>::Graph<Seed>: Clone,
    <Local as Domain<N>>::Fold<H, R>: Clone,
{
    type Seed = Seed;

    fn slice_run_inputs<CurN: Clone + 'static>(seeds: &[Seed], heap: H) -> Self::RunInputs<'static, CurN> {
        let owned: Vec<Seed> = seeds.to_vec();
        let es: local_edgy::Edgy<(), Seed> = local_edgy::edgy_visit(
            move |_: &(), cb: &mut dyn FnMut(&Seed)| {
                for s in &owned {
                    cb(s);
                }
            },
        );
        (es, heap)
    }
}
