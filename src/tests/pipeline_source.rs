//! TreeishSource + PipelineExec on SeedPipeline (the seedless /
//! no-SeedLift path). The seeded / SeedLift path is tested via
//! `Stage2Pipeline::.run(...)` in other test files.

use std::sync::Arc;
use crate::{SeedPipeline, TreeishSource, PipelineExec};
use hylic::domain::shared::{self as dom, fold::fold};
use hylic::exec::funnel;
use hylic::graph::edgy_visit;

fn basic_pipeline() -> SeedPipeline<hylic::domain::Shared, u64, u64, u64, u64> {
    let ch: Arc<Vec<Vec<u64>>> = Arc::new(vec![vec![1, 2], vec![3], vec![], vec![]]);
    let base_fold = fold(|n: &u64| *n, |h: &mut u64, c: &u64| *h += c, |h: &u64| *h);
    let seeds = edgy_visit(move |n: &u64, cb: &mut dyn FnMut(&u64)| {
        if let Some(kids) = ch.get(*n as usize) {
            for k in kids { cb(k); }
        }
    });
    SeedPipeline::new(|s: &u64| *s, seeds, &base_fold)
}

#[test]
fn with_treeish_yields_pair() {
    // SeedPipeline implements TreeishSource by fusing grow +
    // seeds_from_node into a single Graph<N>. Seedless; grow is
    // not exposed.
    let r: u64 = basic_pipeline()
        .with_treeish(|treeish, fold| {
            dom::FUSED.run(&fold, &treeish, &0u64)
        });
    assert_eq!(r, 6);
}

#[test]
fn run_from_node_bypasses_seedlift() {
    // PipelineExec (blanket on TreeishSource) runs the seedless
    // path directly on a SeedPipeline. SeedLift is not involved;
    // there's no Entry variant in sight.
    let r = PipelineExec::run_from_node(
        &basic_pipeline(),
        &dom::exec(funnel::Spec::default(4)),
        &0u64,
    );
    assert_eq!(r, 6);
}
