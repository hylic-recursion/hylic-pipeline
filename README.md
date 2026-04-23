# hylic-pipeline

Typestate pipelines and sugar traits over
[`hylic`](../hylic/)'s lift primitives.

## Layering

`hylic-pipeline` sits on top of `hylic` (core). If you only need
bare lifts (`LiftBare::run_on(&exec, treeish, fold, &root)`), depend
on `hylic` alone. If you want pipeline typestates with chainable
sugars (`.wrap_init(…).zipmap(…)`), depend on `hylic-pipeline`.

```
┌───────────────────────────────────────────┐
│ hylic-pipeline                            │
│   SeedPipeline / TreeishPipeline /         │
│   LiftedPipeline / OwnedPipeline           │
│   TreeishSource / SeedSource               │
│   PipelineExec(Seed / Once)                │
│   LiftedSugarsShared / LiftedSugarsLocal   │
├───────────────────────────────────────────┤
│ hylic  (core)                              │
│   Domain (Shared / Local / Owned)          │
│   Fold / Edgy / Executor                   │
│   Lift / ShapeLift / SeedLift / LiftBare   │
│   ShapeCapable / PureLift / ShareableLift  │
│   Shared::wrap_init_lift, ::n_lift, …      │
└───────────────────────────────────────────┘
```

## Quick use

```rust
use hylic_pipeline::prelude::*;

let pipeline = SeedPipeline::new(
    |s: &u32| *s,
    edgy_visit(|n: &u32, cb: &mut dyn FnMut(&u32)| { /* children */ }),
    &fold(/* init, acc, fin over u32 */),
);

let r = pipeline
    .wrap_init(|n, orig| orig(n) + 1)      // auto-lifts (no .lift() ceremony)
    .zipmap(|r| r > 10)
    .run_from_slice(&exec(funnel::Spec::default(4)), &[0u32], 0u64);
```

## Sugar-trait import

`LiftedSugarsShared` (and `LiftedSugarsLocal` for the Local variant)
provide chainable sugar methods on any pipeline. `use
hylic_pipeline::prelude::*;` brings them in.

See `examples/prelude_minimal.rs` for the smallest complete example.

## Testing

From the workspace root, `make test` runs the lib tests of every
workspace member (306 total incl. 72 hylic-pipeline). `make
bench-integration` opts into the `#[ignore]`-marked benchmark
integration tests in hylic-benchmark.

## History

Split out of `hylic` on 2026-04-23 — pipelines and their sugars
moved to this crate; the lift primitives (`Lift`, `ShapeLift`,
`SeedLift`, `LiftBare`, shape-lift constructors per domain) stayed
in core. Rationale: lifts are library primitives usable without a
pipeline; pipelines add typestate + fluent sugars on top.

See `KB/.plans/finishing-up/crate-split/PLAN.md` in the `hylic`
repo for the detailed design.
