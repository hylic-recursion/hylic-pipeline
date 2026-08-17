//! Fibonacci as a tree fold.
//!
//! The treeish expands `n` into two children `n - 1` and `n - 2`;
//! the fold adds their results. The base recursion is naïve
//! (exponential in `n`), so `n` is kept small.
//!
//! `memoize_by` caches each node's *child enumeration* by key. It
//! reduces graph-function calls when the same `n` is reached along
//! multiple paths, but it does **not** short-circuit the fold walk
//! — the executor still visits every node of the expanded tree.
//! For Fibonacci this means memoisation saves closure calls but
//! not fold operations.
//!
//! Run: `cargo run --example fibonacci -p hylic-pipeline`

use hylic_pipeline::prelude::*;

fn main() {
    let children = treeish(|n: &u64| match *n {
        0 | 1 => vec![],
        k => vec![k - 1, k - 2],
    });

    let add = fold(
        |n: &u64| if *n <= 1 { *n } else { 0 }, // leaves carry their value
        |acc: &mut u64, child: &u64| *acc += child,
        |acc: &u64| *acc,
    );

    let pipeline: TreeishPipeline<Shared, u64, u64, u64> = TreeishPipeline::new(children, &add);

    // Sequential, naïve.
    let r_seq: u64 = pipeline.clone().run_from_node(&FUSED, &15);
    println!("fib(15) sequential         = {r_seq}");
    assert_eq!(r_seq, 610);

    // Sequential with memoised children enumeration. Same result,
    // fewer calls into the graph closure.
    let r_memo: u64 = pipeline
        .clone()
        .memoize_by(|n: &u64| *n)
        .run_from_node(&FUSED, &20);
    println!("fib(20) memoised (Fused)   = {r_memo}");
    assert_eq!(r_memo, 6765);

    // Parallel, naïve. The same pipeline runs unchanged.
    let r_par: u64 = pipeline.run_from_node(&exec(funnel::Spec::default(4)), &15);
    println!("fib(15) parallel (Funnel)  = {r_par}");
    assert_eq!(r_par, 610);
}
