//! `.run` impls for seed-rooted Stage-2 pipelines, plus the
//! GAT-pinning helpers they share. Treeish-rooted chains run via the
//! blanket [`crate::source::PipelineExec`] impl on `TreeishSource` —
//! they need no per-domain run module.

pub mod seed_shared;
pub mod seed_local;
pub(crate) mod gat_helpers;
