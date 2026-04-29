# hylic-pipeline

Typestate pipelines and sugar traits over
[`hylic`](../hylic/)'s lift primitives.

## Layering

`hylic-pipeline` sits on top of `hylic` (core). If you only need
bare lifts (`LiftBare::run_on(&exec, treeish, fold, &root)`), depend
on `hylic` alone. If you want pipeline typestates with chainable
sugars (`.wrap_init(…).zipmap(…)`), depend on `hylic-pipeline`.

```
┌──────────────────────────────────────────────────┐
│ hylic-pipeline                                   │
│   SeedPipeline / TreeishPipeline /                │
│   Stage2Pipeline<Base, L> / OwnedPipeline         │
│   TreeishSource / Stage2Base / Stage2BaseSlice    │
│   PipelineExec / PipelineExecOnce                 │
│   SeedSugars{Shared,Local} /                      │
│   TreeishSugars{Shared,Local} /                   │
│   Stage2Sugars{Shared,Local} (Wrap-dispatched)    │
├──────────────────────────────────────────────────┤
│ hylic  (core)                                     │
│   Domain (Shared / Local / Owned)                 │
│   Fold / Edgy / Executor                          │
│   Lift / ShapeLift / SeedLift / LiftBare          │
│   IdentityLift / ComposedLift / SeedNode          │
│   ShapeCapable / PureLift / ShareableLift         │
│   Shared::wrap_init_lift, ::n_lift, …             │
└──────────────────────────────────────────────────┘
```

## Quick use

```rust
use hylic_pipeline::prelude::*;

let pipeline = SeedPipeline::new(
    |s: &u32| *s,
    edgy_visit(|n: &u32, cb: &mut dyn FnMut(&u32)| { /* children */ }),
    &fold(/* init, acc, fin over u32 */),
);

// No-sugar shorthand on SeedPipeline — empty .lift() elided:
let r = pipeline.run_from_slice(&FUSED, &[0u32], 0u64);

// With Stage-2 sugars — .lift() is the explicit Stage-1 → Stage-2
// transition:
let r = pipeline
    .lift()
    .wrap_init(|n, orig| orig(n) + 1)
    .zipmap(|r| r > 10)
    .run_from_slice(&exec(funnel::Spec::default(4)), &[0u32], 0u64);
```

## Sugar-trait import

`Stage2SugarsShared` (and `Stage2SugarsLocal` for the Local variant)
provide chainable sugar methods on every `Stage2Pipeline<Base, L>`,
dispatched through `Wrap` so one body covers both treeish-rooted and
seed-rooted bases. `use hylic_pipeline::prelude::*;` brings them in.

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
