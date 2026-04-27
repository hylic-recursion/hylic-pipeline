//! Regression: `memoize_by` under the Fused (sequential) executor
//! used to deadlock because `memoize_by_lift` held its cache mutex
//! across the child-visit callback. Fused's direct recursion then
//! re-entered the same memoized graph while the lock was held.
//!
//! This test runs `memoize_by` on a diamond DAG under both Fused and
//! Funnel to confirm the scoped-borrow fix covers both regimes.

#[allow(unused_imports)]
use crate::Stage2SugarsShared;
use std::collections::HashMap;
use std::sync::Arc;
use crate::{TreeishPipeline, PipelineExec, LiftedSugarsShared};
use hylic::domain::shared::{self as dom, fold::fold};
use hylic::exec::funnel;
use hylic::graph::treeish;
use hylic::domain::Shared;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct NodeId(u32);

fn diamond_pipeline() -> TreeishPipeline<Shared, NodeId, u32, u32> {
    let mut m: HashMap<NodeId, Vec<NodeId>> = HashMap::new();
    m.insert(NodeId(0), vec![NodeId(1), NodeId(2)]);
    m.insert(NodeId(1), vec![NodeId(3)]);
    m.insert(NodeId(2), vec![NodeId(3)]);
    m.insert(NodeId(3), vec![NodeId(4)]);
    m.insert(NodeId(4), vec![]);
    let m = Arc::new(m);

    let t = treeish(move |n: &NodeId| m.get(n).cloned().unwrap_or_default());
    let f = fold(
        |_n: &NodeId| 1u32,
        |h: &mut u32, c: &u32| *h += *c,
        |h: &u32| *h,
    );
    TreeishPipeline::new(t, &f)
}

#[test]
fn memoize_by_under_fused_terminates() {
    let r: u32 = diamond_pipeline()
        .lift()
        .memoize_by(|n: &NodeId| n.0)
        .run_from_node(&dom::FUSED, &NodeId(0));
    // Root + {1, 2} + {3, 3} + {4, 4} = 7 finalize visits in the
    // fold tree (memoize dedupes the child *enumeration* but not
    // the per-node fold work; the numbers differ from the naive
    // counter-diamond test but the important thing is that the run
    // terminates at all).
    assert_eq!(r, 7);
}

#[test]
fn memoize_by_under_funnel_terminates() {
    let r: u32 = diamond_pipeline()
        .lift()
        .memoize_by(|n: &NodeId| n.0)
        .run_from_node(&dom::exec(funnel::Spec::default(4)), &NodeId(0));
    assert_eq!(r, 7);
}
