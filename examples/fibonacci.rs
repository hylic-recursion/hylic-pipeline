//! Fibonacci as a tree fold.
//!
//! The treeish expands `n` into two children `n-1` and `n-2`; the
//! fold adds child results. The obvious naïve recursion is
//! exponential in `n`; `memoize_by` on the pipeline makes it
//! linear, and demonstrates a Stage-2 sugar in isolation.
//!
//! Run: `cargo run --example fibonacci -p hylic-pipeline`

use hylic_pipeline::prelude::*;

fn main() {
    // Base recursion: fib(n) = fib(n - 1) + fib(n - 2); fib(0) = 0; fib(1) = 1.
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

    // Without memoisation — exponential, use small n.
    let naive: u64 = pipeline.clone().run_from_node(&FUSED, &10);
    println!("fib(10) naive        = {naive}");
    assert_eq!(naive, 55);

    // With memoisation on the node value (N = u64) — subtree results
    // cached; safely handles fib(40) and beyond.
    let memoised: u64 = pipeline.clone()
        .memoize_by(|n: &u64| *n)
        .run_from_node(&FUSED, &40);
    println!("fib(40) memoised     = {memoised}");
    assert_eq!(memoised, 102_334_155);

    // Under the parallel Funnel executor. For this workload the sum
    // itself is trivial, so Funnel is dominated by scheduling; the
    // point is to show the same pipeline runs unchanged in parallel.
    let parallel: u64 = pipeline
        .memoize_by(|n: &u64| *n)
        .run_from_node(&exec(funnel::Spec::default(4)), &40);
    println!("fib(40) funnel(4)    = {parallel}");
    assert_eq!(parallel, 102_334_155);
}
