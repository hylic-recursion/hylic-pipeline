//! Stage-2 composition matrix.
//!
//! Exercises the exact post-Phase-7 sugar compositions that the seed-
//! pipeline-unification plan called out as load-bearing. Each test runs
//! to completion and pins the chain-tip type so a regression in
//! `Wrap`-driven dispatch (or sugar method selection) shows up as a
//! type error, not a silent semantic drift.

use std::sync::Arc;
use crate::SeedPipeline;
use hylic::domain::shared::{self as dom, fold::fold};
use hylic::domain::Shared;
use hylic::exec::funnel;
use hylic::graph::edgy_visit;
use hylic::ops::SeedNode;
use hylic::prelude::ExplainerResult;

fn basic() -> SeedPipeline<Shared, u64, u64, u64, u64> {
    let ch: Arc<Vec<Vec<u64>>> = Arc::new(vec![vec![1, 2], vec![3], vec![], vec![]]);
    let base_fold = fold(|n: &u64| *n, |h: &mut u64, c: &u64| *h += c, |h: &u64| *h);
    let seeds = edgy_visit(move |n: &u64, cb: &mut dyn FnMut(&u64)| {
        if let Some(kids) = ch.get(*n as usize) { for k in kids { cb(k); } }
    });
    SeedPipeline::new(|s: &u64| *s, seeds, &base_fold)
}

// ── pre-explain composition: wrap_init then explain ─────────

#[test]
fn wrap_init_then_explain() {
    let r: ExplainerResult<SeedNode<u64>, u64, u64> = basic()
        .lift()
        .wrap_init(|n: &u64, orig: &dyn Fn(&u64) -> u64| orig(n) + 1)
        .explain()
        .run_from_slice(&dom::exec(funnel::Spec::default(4)), &[0u64], 0u64);
    // base sum = 6; +1 per real Node init (4 nodes: 0, 1, 2, 3) =>
    // 0 → 1+[6,3]=10; 1 → 2+[4]=6; 2 → 3; 3 → 4; entry → 0+[10]=10.
    assert_eq!(r.orig_result, 10);
    assert!(!r.heap.transitions.is_empty());
}

// ── post-explain composition: wrap_finalize ─────────────────

#[test]
fn explain_then_wrap_finalize() {
    // After explain, R is ExplainerResult<SeedNode<u64>, u64, u64>.
    // wrap_finalize lets us decorate the finalize pass without
    // touching N — the chain-tip R type is unchanged.
    let r: ExplainerResult<SeedNode<u64>, u64, u64> = basic()
        .lift()
        .explain()
        .wrap_finalize(|h, orig| orig(h))
        .run_from_slice(&dom::exec(funnel::Spec::default(4)), &[0u64], 0u64);
    assert_eq!(r.orig_result, 6);
}

// ── post-explain R-bijection ────────────────────────────────

#[test]
fn explain_then_map_r_bi_str() {
    // After explain, R is ExplainerResult<SeedNode<u64>, u64, u64>;
    // map_r_bi swaps it for orig_result-only-strings.
    let r: String = basic()
        .lift()
        .explain()
        .map_r_bi(
            |er: &ExplainerResult<SeedNode<u64>, u64, u64>| format!("orig={}", er.orig_result),
            |s: &String| {
                let n: u64 = s.strip_prefix("orig=").unwrap().parse().unwrap();
                ExplainerResult {
                    heap: hylic::prelude::ExplainerHeap {
                        node: SeedNode::entry_root(),
                        initial_heap: 0,
                        working_heap: 0,
                        transitions: Vec::new(),
                    },
                    orig_result: n,
                }
            },
        )
        .run_from_slice(&dom::exec(funnel::Spec::default(4)), &[0u64], 0u64);
    assert_eq!(r, "orig=6");
}

// ── post-explain wrap_init: dispatch over &SeedNode<N> ──────

#[test]
fn explain_then_wrap_init_dispatches_node_vs_entry() {
    // After explain, the chain-tip N is SeedNode<u64>. The seed-rooted
    // wrap_init sugar dispatches: real Nodes pass through user closure
    // taking &u64; EntryRoot bypasses the user closure (uses orig).
    let counter = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let counter_for_closure = counter.clone();
    let r: ExplainerResult<SeedNode<u64>, u64, u64> = basic()
        .lift()
        .explain()
        .wrap_init(move |_n: &u64,
                    orig: &dyn Fn(&u64) -> hylic::prelude::ExplainerHeap<
                                            SeedNode<u64>, u64, ExplainerResult<SeedNode<u64>, u64, u64>>|
              -> hylic::prelude::ExplainerHeap<
                    SeedNode<u64>, u64, ExplainerResult<SeedNode<u64>, u64, u64>>
            {
                counter_for_closure.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                orig(_n)
            })
        .run_from_slice(&dom::exec(funnel::Spec::default(4)), &[0u64], 0u64);
    // The user-closure runs once per real Node (4 of them). EntryRoot
    // is bypassed by the seed sugar dispatch.
    assert_eq!(counter.load(std::sync::atomic::Ordering::Relaxed), 4);
    assert_eq!(r.orig_result, 6);
}

// ── nested explain ──────────────────────────────────────────

#[test]
fn explain_then_explain_nests_traces() {
    // The outer explain wraps the inner explain's R as its R. The
    // chain-tip type pin is verbose but sound: we just confirm the
    // outer trace's orig_result equals the inner trace's orig_result.
    let r: ExplainerResult<
        SeedNode<u64>,
        hylic::prelude::ExplainerHeap<SeedNode<u64>, u64, ExplainerResult<SeedNode<u64>, u64, u64>>,
        ExplainerResult<SeedNode<u64>, u64, u64>,
    > = basic()
        .lift()
        .explain()
        .explain()
        .run_from_slice(&dom::exec(funnel::Spec::default(4)), &[0u64], 0u64);
    // Outer trace's orig_result is the inner trace's orig_result-shaped value.
    assert_eq!(r.orig_result.orig_result, 6);
}

// ── long composition: explain → zipmap → map_r_bi ───────────

#[test]
fn explain_then_zipmap_then_map_r_bi() {
    let r: u64 = basic()
        .lift()
        .explain()
        .zipmap(|er: &ExplainerResult<SeedNode<u64>, u64, u64>| er.orig_result + 100)
        .map_r_bi(
            |pair: &(ExplainerResult<SeedNode<u64>, u64, u64>, u64)| pair.1,
            |n: &u64| {
                (
                    ExplainerResult {
                        heap: hylic::prelude::ExplainerHeap {
                            node: SeedNode::entry_root(),
                            initial_heap: 0,
                            working_heap: 0,
                            transitions: Vec::new(),
                        },
                        orig_result: *n,
                    },
                    *n,
                )
            },
        )
        .run_from_slice(&dom::exec(funnel::Spec::default(4)), &[0u64], 0u64);
    // zipmap is applied per-node uniformly. At each finalize:
    //   leaf            extra = orig + 100
    //   parent          orig accumulates children's RNew (their extras),
    //                   so each level compounds with another +100.
    // Tree height from entry-root: entry→node0→node1→node3 = 4 levels;
    // entry's extra = base_sum (6) + 5*100 = 506.
    assert_eq!(r, 506);
}
