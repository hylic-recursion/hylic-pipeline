# hylic-pipeline

Chainable typestate builder over
[`hylic`](https://github.com/hylic-recursion/hylic). Each method
either reshapes a pipeline's stage-1 slots (graph, fold, seeds,
grow function) or composes a lift onto a stage-2 chain. The
typestate enforces the order in which transforms can apply.

```rust
use hylic_pipeline::prelude::*;

let result = SeedPipeline::new(grow, edges, &fold)
    .filter_seeds(|s| !s.is_empty())
    .wrap_init(|node, orig| orig(node) + 1)
    .zipmap(|r| *r > 10)
    .run_from_slice(&exec(funnel::Spec::default(4)), &seeds, heap);
```

The book has the full typestate diagram, the catalogue of
sugars, and a guide to writing custom lifts:
<https://hylic-recursion.github.io/hylic-docs/pipeline/overview.html>.

If you only need bare lifts (`LiftBare::run_on`), `hylic` alone
is sufficient — this crate is the convenience surface, not the
foundation.

## License

Licensed under the [MIT License](./LICENSE).
