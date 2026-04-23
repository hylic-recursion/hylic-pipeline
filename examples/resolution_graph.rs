//! Dependency-graph resolution driven by a `SeedPipeline`.
//!
//! The tree is not known in advance: module names (`Seed`, a
//! `String`) resolve into `Module` records via a registry, and each
//! resolved module declares the names of its dependencies. The
//! pipeline alternates `grow` and `seeds_from_node` until every
//! reachable dependency has been materialised.
//!
//! The fold, meanwhile, counts the total number of modules in the
//! transitive closure.
//!
//! Run: `cargo run --example resolution_graph -p hylic-pipeline`

use std::collections::HashMap;

use hylic_pipeline::prelude::*;
use hylic::graph::{Edgy, edgy_visit};

#[derive(Clone, Debug)]
struct Module {
    name: String,
    deps: Vec<String>,
}

fn main() {
    // The registry — in a real system this would be a filesystem
    // scan, a lockfile, an HTTP API, etc.
    let registry: HashMap<String, Module> = [
        ("app",  vec!["db", "http", "log"]),
        ("db",   vec!["log"]),
        ("http", vec!["tls", "log"]),
        ("tls",  vec!["log"]),
        ("log",  vec![] as Vec<&'static str>),
    ]
    .into_iter()
    .map(|(name, deps)| (
        name.to_string(),
        Module { name: name.to_string(), deps: deps.iter().map(|s| s.to_string()).collect() },
    ))
    .collect();

    // Grow: resolve a name to a Module record.
    let reg = registry.clone();
    let grow = move |seed: &String| -> Module {
        reg.get(seed)
            .cloned()
            .unwrap_or_else(|| panic!("unresolved dependency: {seed}"))
    };

    // seeds_from_node: a Module yields its dependencies as next-level seeds.
    let children: Edgy<Module, String> =
        edgy_visit(|m: &Module, cb: &mut dyn FnMut(&String)| {
            for dep in &m.deps { cb(dep); }
        });

    // Fold: count modules (1 per node plus each child's subtree count),
    // deduplication happens at the level of memoisation if desired.
    let count = fold(
        |_m: &Module| 1_u64,
        |acc: &mut u64, child_total: &u64| *acc += child_total,
        |acc: &u64| *acc,
    );

    // Assemble the pipeline.
    let pipeline: SeedPipeline<Shared, Module, String, u64, u64> =
        SeedPipeline::new(grow, children, &count);

    // Memoise on module name so each module is counted once even when
    // it is reachable from multiple dependents (shared `log`, here).
    let total: u64 = pipeline
        .memoize_by(|m: &Module| m.name.clone())
        .run_from_slice(&FUSED, &["app".to_string()], 0);

    println!("transitive module count from 'app' = {total}");
    // app + db + http + tls + log = 5 unique modules.
    assert_eq!(total, 5);
}
