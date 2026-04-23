//! Stage-1 blanket sugar for `TreeishPipeline<Local, …>`. Mirror of
//! `treeish_shared.rs` with Rc storage and no Send+Sync bounds.

#![allow(missing_docs)] // module-level: public items are per-domain/per-policy mirrors of documented primitives

use std::rc::Rc;
use hylic::domain::Local;
use hylic::domain::local::Fold;
use hylic::domain::local::edgy::Edgy;
use crate::treeish::TreeishPipeline;

pub trait TreeishSugarsLocal<N, H, R>: Sized
where N: Clone + 'static, H: Clone + 'static, R: Clone + 'static,
{
    fn map_node_bi<N2, Co, Contra>(self, co: Co, contra: Contra)
        -> TreeishPipeline<Local, N2, H, R>
    where N2: Clone + 'static,
          Co:     Fn(&N) -> N2 + 'static,
          Contra: Fn(&N2) -> N + 'static;
}

impl<N, H, R> TreeishSugarsLocal<N, H, R> for TreeishPipeline<Local, N, H, R>
where N: Clone + 'static, H: Clone + 'static, R: Clone + 'static,
{
    fn map_node_bi<N2, Co, Contra>(self, co: Co, contra: Contra)
        -> TreeishPipeline<Local, N2, H, R>
    where N2: Clone + 'static,
          Co:     Fn(&N) -> N2 + 'static,
          Contra: Fn(&N2) -> N + 'static,
    {
        let co = Rc::new(co);
        let contra = Rc::new(contra);
        let co_for_treeish = co.clone();
        let contra_for_treeish = contra.clone();
        let contra_for_fold = contra.clone();
        self.reshape(
            move |treeish: Edgy<N, N>| -> Edgy<N2, N2> {
                treeish.contramap(move |n2: &N2| contra_for_treeish(n2))
                       .map(move |n: &N| co_for_treeish(n))
            },
            move |fold: Fold<N, H, R>| -> Fold<N2, H, R> {
                fold.contramap_n(move |n2: &N2| contra_for_fold(n2))
            },
        )
    }
}
