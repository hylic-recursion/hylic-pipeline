# hylic-pipeline

Chainable typestate builder over [`hylic`](https://github.com/hylic-recursion/hylic).
Where `hylic` gives you `Fold`, `Treeish`, and `Executor` as
values you compose by hand, `hylic-pipeline` packs them into a
pipeline that you build by chaining methods.

The typestate enforces the order in which transforms can
apply. Stage-1 methods reshape the slot values (graph, fold,
seeds, grow function); `.lift()` flips into stage 2, where
every method composes a lift onto the chain. Underneath, the
result is the same fold-treeish-executor decomposition; the
ergonomic gain is at construction.

```rust
use hylic_pipeline::prelude::*;

let result = SeedPipeline::new(grow, edges, &fold)
    .filter_seeds(|s| !s.is_empty())          // stage 1: reshape seeds
    .wrap_init(|node, orig| orig(node) + 1)    // stage 2: wrap fold init
    .zipmap(|r| *r > 10)                        // stage 2: extend R
    .run_from_slice(&exec(funnel::Spec::default(4)), &seeds, heap);
```

## Reading order for newcomers

- **[Pipeline overview](https://hylic-recursion.github.io/hylic-docs/pipeline/overview.html)** — when to reach for a pipeline vs bare lifts, the stage-1 / stage-2 typestate explained.
- **[Sugar catalogue](https://hylic-recursion.github.io/hylic-docs/pipeline/sugars.html)** — every chainable method, what it does, where it applies.
- **[SeedPipeline](https://hylic-recursion.github.io/hylic-docs/pipeline/seed.html)** and **[TreeishPipeline](https://hylic-recursion.github.io/hylic-docs/pipeline/treeish.html)** — the two stage-1 starting points.
- **[Stage2Pipeline](https://hylic-recursion.github.io/hylic-docs/pipeline/lifted.html)** — the unified stage-2 type both stage-1 bases flow into.
- **[Writing a custom lift](https://hylic-recursion.github.io/hylic-docs/pipeline/custom_lift.html)** — when the catalogue isn't enough.

## Related crates

- [`hylic`](https://github.com/hylic-recursion/hylic) — core.
  The fold / treeish / executor primitives this crate is
  built on.
- [`hylic-docs`](https://github.com/hylic-recursion/hylic-docs)
  — mdBook source for the documentation site above. Not on
  crates.io.

## License

Licensed under the [MIT License](./LICENSE).
