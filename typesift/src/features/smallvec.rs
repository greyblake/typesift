//! `smallvec`, behind the `smallvec` feature.
//!
//! A `SmallVec` is walked like a `Vec`: it is offered itself, then each item, whether the items sit
//! inline or have spilled to the heap.

use std::ops::ControlFlow;

use smallvec::{Array, SmallVec};

use crate::TypeSift;

impl<A: Array + 'static> TypeSift for SmallVec<A>
where
    A::Item: TypeSift,
{
    fn visit<'a, T: 'static, B, F>(&'a self, visitor: &mut F) -> ControlFlow<B>
    where
        F: FnMut(&'a T) -> ControlFlow<B>,
    {
        self.visit_self::<T, B, F>(visitor)?;
        self.iter()
            .try_for_each(|item| item.visit::<T, B, F>(visitor))
    }
}
