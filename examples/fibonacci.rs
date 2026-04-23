//! Fibonacci as a naïve tree fold.
//!
//! The treeish expands `n` into two children `n - 1` and `n - 2`;
//! the fold adds their results. The recursion is deliberately naïve
//! (exponential in `n`) — its purpose is to demonstrate the pipeline
//! shape under both executors rather than to compute Fibonacci
//! efficiently.
//!
//! A note on memoisation: `memoize_by` caches the *children* of a
//! node, not the *fold result*, so it does not collapse the
//! exponential recursion. For real memoisation the caller must
//! fold over a DAG whose shared nodes are already collapsed in the
//! tree structure (see `examples/resolution_graph.rs`).
//!
//! Run: `cargo run --example fibonacci -p hylic-pipeline`

use hylic_pipeline::prelude::*;

fn main() {
    let children = treeish(|n: &u64| match *n {
        0 | 1 => vec![],
        k     => vec![k - 1, k - 2],
    });

    let add = fold(
        |n: &u64| if *n <= 1 { *n } else { 0 }, // leaves carry their value
        |acc: &mut u64, child: &u64| *acc += child,
        |acc: &u64| *acc,
    );

    let pipeline: TreeishPipeline<Shared, u64, u64, u64> =
        TreeishPipeline::new(children, &add);

    // Sequential (Fused) — small n to keep the run cheap.
    let r_seq: u64 = pipeline.clone().run_from_node(&FUSED, &15);
    println!("fib(15) sequential = {r_seq}");
    assert_eq!(r_seq, 610);

    // Parallel (Funnel) — same pipeline, different executor.
    let r_par: u64 = pipeline.run_from_node(&exec(funnel::Spec::default(4)), &15);
    println!("fib(15) parallel   = {r_par}");
    assert_eq!(r_par, 610);
}
