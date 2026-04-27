//! ExplainerDescribe: streaming per-node trace emission with
//! transparent R. Under Option B the chain is typed at
//! `SeedNode<N>`; Entry is a first-class value of the node type.

#[allow(unused_imports)]
use crate::Stage2SugarsShared;
use std::sync::{Arc, Mutex};
use crate::SeedPipeline;
use hylic::domain::shared::{self as dom, fold::fold};
use hylic::exec::funnel;
use hylic::domain::Shared;
use hylic::graph::edgy_visit;
use hylic::ops::SeedNode;
use hylic::prelude::trace_fold_compact;

fn basic_pipeline() -> SeedPipeline<Shared, u64, u64, u64, u64> {
    let ch: Arc<Vec<Vec<u64>>> = Arc::new(vec![vec![1, 2], vec![3], vec![], vec![]]);
    let base_fold = fold(
        |n: &u64| *n,
        |h: &mut u64, c: &u64| *h += c,
        |h: &u64| *h,
    );
    let seeds = edgy_visit(move |n: &u64, cb: &mut dyn FnMut(&u64)| {
        if let Some(kids) = ch.get(*n as usize) { for k in kids { cb(k); } }
    });
    SeedPipeline::new(|s: &u64| *s, seeds, &base_fold)
}

#[test]
fn explainer_describe_streams_per_node_and_preserves_r() {
    let captured: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let captured_for_emit = captured.clone();

    // trace_fold_compact renders `heap.node`; at the chain's
    // seed-closed level the node type is SeedNode<u64>.
    let r: u64 = basic_pipeline()
        .lift()
        .explain_describe(
            trace_fold_compact::<SeedNode<u64>, u64, u64>,
            move |s: &str| {
                captured_for_emit.lock().unwrap().push(s.to_string());
            },
        )
        .run_from_slice(&dom::exec(funnel::Spec::default(4)), &[0u64], 0u64);

    // R unchanged: sum = 0 + 1 + 2 + 3 = 6.
    assert_eq!(r, 6);

    let lines = captured.lock().unwrap();
    assert!(!lines.is_empty(), "trace emitted at least one line");
    for line in lines.iter() {
        assert!(line.contains("=>"), "trace line formatted: {line}");
    }
}
