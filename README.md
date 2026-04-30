# hylic-pipeline

A typestate pipeline builder over the [`hylic`](https://github.com/hylic-recursion/hylic) primitives. The pipeline holds the same three values `hylic` would otherwise compose by hand — a graph, a fold, and (for seed-rooted recursions) a `grow: Seed -> N` function — and exposes them through a chained-method API that the type system orders.

There are two stage-1 starting points. `TreeishPipeline<D, N, H, R>` is the direct case: children come from nodes of the same type, a recursion runs from a root `&N`. `SeedPipeline<D, N, Seed, H, R>` is the lazy case: children are referenced by `Seed` values that get resolved through `grow` on demand, and a recursion runs from a slice of entry seeds. Both have stage-1 sugars that reshape the slot values themselves: `filter_seeds`, `wrap_grow`, `map_node_bi`, `map_seed_bi` on the seed side, `map_node_bi` on the treeish side.

`.lift()` flips a stage-1 pipeline into a `Stage2Pipeline<Base, L>`. From there every chained method appends a `Lift` to `L` (a `ComposedLift<L1, L2>` tree built up by the type system). The Stage-2 sugar surface is what users will recognise: `wrap_init`, `wrap_accumulate`, `wrap_finalize`, `zipmap`, `map_r_bi`, `filter_edges`, `memoize_by`, `explain`, plus `then_lift` and `before_lift` for direct lift composition. `TreeishPipeline` auto-lifts on the first stage-2 call; `SeedPipeline` requires `.lift()` explicitly because the chain's input type changes from `N` to `SeedNode<N>` at the boundary.

```rust
use hylic_pipeline::prelude::*;

let result = SeedPipeline::new(grow, edges, &fold)
    .filter_seeds(|s| !s.is_empty())          // stage 1: reshape seeds
    .wrap_init(|node, orig| orig(node) + 1)    // stage 2: wrap fold init
    .zipmap(|r| *r > 10)                        // stage 2: extend R
    .run_from_slice(&exec(funnel::Spec::default(4)), &seeds, heap);
```

`hylic_pipeline::prelude::*` re-exports `hylic::prelude::*`; one import covers both crates.

The whole chain monomorphises. Stage-2 sugars produce a `ShapeLift` (the universal shape-changing lift type) under the hood, then call `then_lift` on the chain; there is no runtime dispatch on which sugar was called. The compiled walk is the same one a hand-written `wrap_init_lift(...).then_lift(zipmap_lift(...)).run_on(&exec, treeish, fold, &root)` would produce.

## Documentation

The [pipeline overview](https://hylic-recursion.github.io/hylic-docs/pipeline/overview.html) explains the stage-1/stage-2 typestate, when a pipeline applies versus the bare hylic API, and what each stage contributes. The [sugar catalogue](https://hylic-recursion.github.io/hylic-docs/pipeline/sugars.html) lists every chainable method, the stage in which it applies, and what it does to the chain's `(N, H, R)`. [SeedPipeline](https://hylic-recursion.github.io/hylic-docs/pipeline/seed.html) and [TreeishPipeline](https://hylic-recursion.github.io/hylic-docs/pipeline/treeish.html) cover the two starting points. [Stage2Pipeline](https://hylic-recursion.github.io/hylic-docs/pipeline/lifted.html) is the unified stage-2 type both bases flow into. [Writing a custom lift](https://hylic-recursion.github.io/hylic-docs/pipeline/custom_lift.html) describes the trait surface for operations the catalogue does not cover.

## Related crates

[`hylic`](https://github.com/hylic-recursion/hylic) is the core library; the fold, treeish, and executor primitives this crate is built on. [`hylic-docs`](https://github.com/hylic-recursion/hylic-docs) is the mdBook source for the documentation site linked above; not on crates.io.

## License

Licensed under the [MIT License](./LICENSE).
