//! Cookbook: explainer_lift / explainer_describe_lift.

use crate::SeedPipeline;
#[allow(unused_imports)]
use crate::Stage2SugarsShared;
use hylic::domain::Shared;
use hylic::domain::shared::{self as dom, fold::fold};
use hylic::exec::funnel;
use hylic::graph::edgy_visit;
use hylic::ops::SeedNode;
use hylic::prelude::{ExplainerResult, trace_fold_compact};
use std::sync::{Arc, Mutex};

fn basic() -> SeedPipeline<Shared, u64, u64, u64, u64> {
    let ch: Arc<Vec<Vec<u64>>> = Arc::new(vec![
        vec![1, 2],
        vec![3],
        vec![],
        vec![],
    ]);
    let base_fold = fold(
        |n: &u64| *n,
        |h: &mut u64, c: &u64| *h += c,
        |h: &u64| *h,
    );
    let seeds = edgy_visit(
        move |n: &u64, cb: &mut dyn FnMut(&u64)| {
            if let Some(kids) = ch.get(*n as usize) {
                for k in kids {
                    cb(k);
                }
            }
        },
    );
    SeedPipeline::new(|s: &u64| *s, seeds, &base_fold)
}

#[test]
fn explainer_lift_records_full_trace() {
    let r: ExplainerResult<SeedNode<u64>, u64, u64> = basic().lift().explain().run_from_slice(
        &dom::exec(funnel::Spec::default(4)),
        &[0u64],
        0u64,
    );
    assert_eq!(r.orig_result, 6);
    assert!(
        !r.heap.transitions.is_empty(),
        "trace recorded"
    );
}

#[test]
fn explainer_describe_streams_per_node() {
    let captured: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let captured_for_emit = captured.clone();

    let r: u64 = basic()
        .lift()
        .explain_describe(
            trace_fold_compact::<SeedNode<u64>, u64, u64>,
            move |s: &str| {
                captured_for_emit.lock().unwrap().push(s.to_string());
            },
        )
        .run_from_slice(
            &dom::exec(funnel::Spec::default(4)),
            &[0u64],
            0u64,
        );
    // R is transparent.
    assert_eq!(r, 6);
    assert!(!captured.lock().unwrap().is_empty());
}
