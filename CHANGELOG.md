# Changelog

All notable changes to `hylic-pipeline` are documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.8.0] — 2026-05 (pending release)

The first tagged release. `hylic-pipeline` was extracted from
the `hylic` core during early development; this entry records
the pipeline-specific milestones of that evolution.

### Added

- **Pipeline crate extracted from `hylic` core.** Initial import
  of the pipeline subtree (`f55fd44`, `7f677c5`); the core crate
  no longer carries the typestate builder.
- **Stage 1 and Stage 2 typestates.** `SeedPipeline`,
  `TreeishPipeline`, `LiftedPipeline`, `OwnedPipeline` — each
  parametric over `D: Domain<N>` (core-side `ab4c6ff`).
- **`TreeishSource` / `SeedSource` / `PipelineExec` /
  `PipelineExecSeed` traits** consolidating how pipelines expose
  themselves to executors (core-side `7d42bb5`).
- **Blanket sugar traits per stage × domain**: six files under
  `sugars/` define the `.wrap_init(...)`, `.zipmap(...)`,
  `.filter_edges(...)`, `.then_lift(...)` surface once each for
  Shared and Local (core-side `2639c5d`, `80f918f`; pipeline-side
  `b8a4397`).
- **`LiftedPipeline` primitives consolidated** into
  `lifted/primitives.rs` — `then_lift` and `before_lift` are the
  two composition points; every sugar delegates to one of them
  (`301983c`).

### Changed

- **`_local`-suffixed inherent methods dropped** in favour of
  `*Local` blanket traits (`b8a4397`).

  Note: commits tagged `core-side` refer to the `hylic` repository;
  untagged shas are in this (`hylic-pipeline`) repository.
- **Prelude streamlined**: removes the now-obsolete `local` /
  `owned` submodules (`423f497`).
- **`hylic::exec` path adopted** after the core's `cata/ → exec/`
  flattening (`532da01`).

### Notes

Two mirror-file patterns (`sugars/{seed,treeish,lifted}_shared.rs`
vs their `_local` counterparts) are accepted as documented debt;
see the core `hylic` CHANGELOG for the shared rationale.
