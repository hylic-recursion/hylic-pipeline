//! Seed + N-change integration tests.
//!
//! Demonstrates that Stage-2 N-changing lifts compose with
//! `SeedPipeline` via covariant grow transport through
//! `Lift::project_entry_node` (the project-entry refactor).
//!
//! Before the refactor, `LiftedPipeline::SeedSource` carried a
//! `L::N2 = Base::N` constraint that rejected any N-changing lift
//! on the Seed path. After, the forward entry-map transports grow.
//!
//! See: `KB/.plans/project-entry-refactor/` for the derivation.

use std::sync::Arc;

use crate::{SeedPipeline, PipelineExecSeed};
use hylic::domain::{Shared};
use hylic::domain::shared::{self as dom, fold::fold};
use hylic::exec::funnel;
use hylic::graph::edgy_visit;

fn base() -> SeedPipeline<Shared, u64, u64, u64, u64> {
    let ch: Arc<Vec<Vec<u64>>> = Arc::new(vec![
        vec![1, 2], vec![3], vec![], vec![],
    ]);
    let base_fold = fold(|n: &u64| *n, |h: &mut u64, c: &u64| *h += c, |h: &u64| *h);
    let seeds = edgy_visit(move |n: &u64, cb: &mut dyn FnMut(&u64)| {
        if let Some(kids) = ch.get(*n as usize) { for k in kids { cb(k); } }
    });
    SeedPipeline::new(|s: &u64| *s, seeds, &base_fold)
}

#[test]
fn map_n_bi_lift_composes_on_seed_path() {
    // The test that used to be a compile error.
    //
    // Before: `seed_pipeline.lift().then_lift(Shared::map_n_bi_lift(...))
    //         .run_from_slice(...)` was rejected because the SeedSource
    //         impl required L::N2 = Base::N.
    //
    // After: the forward entry-map transports grow covariantly, so
    // N-changing lifts compose on the Seed path.

    #[derive(Clone, Debug, PartialEq)]
    struct Wrapped(u64);

    let lift = Shared::map_n_bi_lift::<u64, u64, u64, Wrapped, _, _>(
        |n: &u64| Wrapped(*n),
        |w: &Wrapped| w.0,
    );

    let r: u64 = base()
        .lift()
        .then_lift(lift)
        .run_from_slice(&dom::exec(funnel::Spec::default(4)), &[0u64], 0u64);

    // Tree from Entry → Node(Wrapped(0)) → {Wrapped(1), Wrapped(2)};
    // Wrapped(1) → {Wrapped(3)}.
    // Fold (through the contramap) sums original u64 values.
    // Subtree(0) = 0 + (1 + 3) + 2 = 6.
    // Entry heap = 0, accumulate child(6) → 6.
    assert_eq!(r, 6);
}

#[test]
fn multi_seed_with_n_change() {
    // Two entry seeds + N-change. Grow transport fans out correctly.
    #[derive(Clone, Debug, PartialEq)]
    struct W(u64);

    let lift = Shared::map_n_bi_lift::<u64, u64, u64, W, _, _>(
        |n: &u64| W(*n),
        |w: &W| w.0,
    );

    let r: u64 = base()
        .lift()
        .then_lift(lift)
        .run_from_slice(&dom::exec(funnel::Spec::default(4)), &[0u64, 1u64], 0u64);

    // subtree(0) = 6, subtree(1) = 1 + 3 = 4. Entry heap = 0.
    // Total = 0 + 6 + 4 = 10.
    assert_eq!(r, 10);
}
