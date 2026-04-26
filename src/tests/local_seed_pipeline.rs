//! Local-domain `LiftedSeedPipeline` — sugars + run. Mirrors a
//! subset of `intuitive_reuse.rs` / `shape_shifting.rs` / `power_user.rs`
//! against Local storage, proving parity with Shared.

use std::rc::Rc;
use crate::SeedPipeline;
use hylic::domain::{local, Local};
use hylic::ops::{SeedNode};
use hylic::prelude::{ExplainerResult, SeedExplainerResult};

fn basic() -> SeedPipeline<Local, u64, u64, u64, u64> {
    let ch: Rc<Vec<Vec<u64>>> = Rc::new(vec![
        vec![1, 2], vec![3], vec![], vec![],
    ]);
    let base_fold = local::fold(
        |n: &u64| *n,
        |h: &mut u64, c: &u64| *h += c,
        |h: &u64| *h,
    );
    let seeds = local::edgy::edgy_visit(move |n: &u64, cb: &mut dyn FnMut(&u64)| {
        if let Some(kids) = ch.get(*n as usize) { for k in kids { cb(k); } }
    });
    SeedPipeline::<Local, u64, u64, u64, u64>::new_local(|s: &u64| *s, seeds, &base_fold)
}

#[test]
fn local_run_from_slice_baseline() {
    let r = basic()
        .lift()
        .run_from_slice(&local::FUSED, &[0u64], 0u64);
    // 0+1+2+3 = 6.
    assert_eq!(r, 6);
}

#[test]
fn local_wrap_init_sugar_composes() {
    let r = basic()
        .lift()
        .wrap_init(|n: &u64, orig: &dyn Fn(&u64) -> u64| orig(n) + 1)
        .run_from_slice(&local::FUSED, &[0u64], 0u64);
    // Each Node's init gets +1. Leaves: 3→4, 2→3. node 1: 2+4=6. node 0: 1+6+3=10. Entry: 0+10=10.
    assert_eq!(r, 10);
}

#[test]
fn local_zipmap_and_map_r_bi() {
    let r: String = basic()
        .lift()
        .zipmap(|r: &u64| *r > 5)
        .map_r_bi(
            |t: &(u64, bool)| format!("{}:{}", t.0, t.1),
            |s: &String| {
                let (a, b) = s.split_once(':').unwrap();
                (a.parse().unwrap(), b == "true")
            },
        )
        .run_from_slice(&local::FUSED, &[0u64], 0u64);
    assert_eq!(r, "6:true");
}

#[test]
fn local_map_n_bi_stage2() {
    // Stage-2 bijective N-change. Demonstrates the new sugar on
    // LiftedSeedPipeline (Local).
    let r: u64 = basic()
        .lift()
        .map_n_bi::<i64, _, _>(|n: &u64| *n as i64, |n: &i64| *n as u64)
        .wrap_init(|n: &i64, orig: &dyn Fn(&i64) -> u64| orig(n) + 10)
        .run_from_slice(&local::FUSED, &[0u64], 0u64);
    // wrap_init adds 10 per node. Per-subtree: 3→13, 2→12, 1→11+13=24, 0→10+24+12=46, Entry→46.
    assert_eq!(r, 46);
}

#[test]
fn local_explain_projects_via_seed_explainer_result() {
    let raw: ExplainerResult<SeedNode<u64>, u64, u64> = basic()
        .lift()
        .explain()
        .run_from_slice(&local::FUSED, &[0u64], 0u64);

    // Raw chain-tip carries SeedNode<N>.
    assert_eq!(raw.orig_result, 6);
    assert!(raw.heap.node.is_entry_root());
    assert_eq!(raw.heap.transitions.len(), 1); // one root seed

    // Sealed projection: N-typed view, no SeedNode.
    let sealed: SeedExplainerResult<u64, u64, u64> = raw.into();
    assert_eq!(sealed.entry_initial_heap, 0);
    assert_eq!(sealed.entry_working_heap, 6);
    assert_eq!(sealed.orig_result, 6);
    assert_eq!(sealed.roots.len(), 1);

    let root = &sealed.roots[0];
    assert_eq!(root.heap.node, 0); // plain u64, no SeedNode wrap
    assert_eq!(root.orig_result, 6);
    assert_eq!(root.heap.transitions.len(), 2); // children 1 and 2
}
