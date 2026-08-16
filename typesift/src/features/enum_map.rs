//! `enum-map`, behind the `enum-map` feature.
//!
//! The values are walked, in the order the enum declares its variants. The keys are not searched,
//! unlike in every other map: an `EnumMap` is an array indexed by the key's position, so it stores
//! no keys to hand out references to. A key type therefore needs no `TypeSift` impl either.

use std::ops::ControlFlow;

use enum_map::{Enum, EnumMap};

use crate::TypeSift;

impl<K: Enum + 'static, V: TypeSift> TypeSift for EnumMap<K, V> {
    fn visit<'a, T: 'static, B, F>(&'a self, visitor: &mut F) -> ControlFlow<B>
    where
        F: FnMut(&'a T) -> ControlFlow<B>,
    {
        self.visit_self::<T, B, F>(visitor)?;
        self.values()
            .try_for_each(|value| value.visit::<T, B, F>(visitor))
    }
}
