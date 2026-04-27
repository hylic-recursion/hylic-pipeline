//! Smoke tests for Stage-2 sugars on SeedPipeline (via LiftedSeedPipeline)
//! and TreeishPipeline (via LiftedPipeline).
//!
//! Under Option B: SeedPipeline requires an explicit `.lift()` to
//! transition to LiftedSeedPipeline before Stage-2 sugars apply.
//! TreeishPipeline retains the blanket auto-lift trait.

#[allow(unused_imports)]
use crate::Stage2SugarsShared;
use std::sync::Arc;
use crate::{
    SeedPipeline, TreeishPipeline, PipelineExec,
    LiftedSugarsShared,
};
use hylic::domain::shared::{self as dom, fold::fold};
use hylic::exec::funnel;
use hylic::domain::Shared;
use hylic::graph::{edgy_visit, treeish};

fn seed_pipeline() -> SeedPipeline<Shared, u64, u64, u64, u64> {
    let ch: Arc<Vec<Vec<u64>>> = Arc::new(vec![vec![1, 2], vec![3], vec![], vec![]]);
    let base_fold = fold(|n: &u64| *n, |h: &mut u64, c: &u64| *h += c, |h: &u64| *h);
    let seeds = edgy_visit(move |n: &u64, cb: &mut dyn FnMut(&u64)| {
        if let Some(kids) = ch.get(*n as usize) { for k in kids { cb(k); } }
    });
    SeedPipeline::new(|s: &u64| *s, seeds, &base_fold)
}

#[test]
fn seed_pipeline_wrap_init_after_lift() {
    let r = seed_pipeline()
        .lift()
        .wrap_init(|n: &u64, orig: &dyn Fn(&u64) -> u64| orig(n) + 1)
        .run_from_slice(&dom::exec(funnel::Spec::default(4)), &[0u64], 0u64);
    // wrap_init(+1) on tree {0→{1,2}, 1→{3}}:
    // 3:4; 1:2+4=6; 2:3; 0:1+6+3=10; Entry:0+10=10.
    assert_eq!(r, 10);
}

#[test]
fn seed_pipeline_chain_after_lift() {
    let r = seed_pipeline()
        .lift()
        .wrap_init(|n: &u64, orig: &dyn Fn(&u64) -> u64| orig(n) + 1)
        .zipmap(|r: &u64| *r > 5)
        .run_from_slice(&dom::exec(funnel::Spec::default(4)), &[0u64], 0u64);
    assert_eq!(r, (10u64, true));
}

#[test]
fn treeish_pipeline_wrap_init_via_trait() {
    let t = treeish(|n: &u64| match *n {
        0 => vec![1, 2],
        1 => vec![3],
        _ => vec![],
    });
    let f = fold(|n: &u64| *n, |h: &mut u64, c: &u64| *h += c, |h: &u64| *h);
    let r: u64 = TreeishPipeline::<Shared, u64, u64, u64>::new(t, &f)
        .wrap_init(|n: &u64, orig: &dyn Fn(&u64) -> u64| orig(n) + 1)
        .run_from_node(&dom::exec(funnel::Spec::default(4)), &0u64);
    assert_eq!(r, 10);
}

#[test]
fn map_r_bi_after_lift() {
    let r: String = seed_pipeline()
        .lift()
        .map_r_bi(
            |r: &u64| format!("sum={r}"),
            |s: &String| s.strip_prefix("sum=").unwrap().parse::<u64>().unwrap(),
        )
        .run_from_slice(&dom::exec(funnel::Spec::default(4)), &[0u64], 0u64);
    assert_eq!(r, "sum=6");
}
