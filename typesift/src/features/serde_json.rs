//! `serde_json`, behind the `serde_json` feature.
//!
//! A `Value` is walked: it is offered itself, then whatever it holds, so a search finds every
//! string or number in a document. A `Map` offers each key before its value. A `Number` is a leaf,
//! because the integer or float inside it cannot be borrowed.

use std::ops::ControlFlow;

use serde_json::{Map, Number, Value};

use crate::{TypeSift, impl_type_sift_for_leaf};

impl_type_sift_for_leaf!(Number);

impl TypeSift for Value {
    fn visit<'a, T: 'static, B, F>(&'a self, visitor: &mut F) -> ControlFlow<B>
    where
        F: FnMut(&'a T) -> ControlFlow<B>,
    {
        self.visit_self::<T, B, F>(visitor)?;
        match self {
            Value::Null => ControlFlow::Continue(()),
            Value::Bool(value) => value.visit::<T, B, F>(visitor),
            Value::Number(number) => number.visit::<T, B, F>(visitor),
            Value::String(text) => text.visit::<T, B, F>(visitor),
            Value::Array(items) => items.visit::<T, B, F>(visitor),
            Value::Object(fields) => fields.visit::<T, B, F>(visitor),
        }
    }
}

impl TypeSift for Map<String, Value> {
    fn visit<'a, T: 'static, B, F>(&'a self, visitor: &mut F) -> ControlFlow<B>
    where
        F: FnMut(&'a T) -> ControlFlow<B>,
    {
        self.visit_self::<T, B, F>(visitor)?;
        self.iter().try_for_each(|(key, value)| {
            key.visit::<T, B, F>(visitor)?;
            value.visit::<T, B, F>(visitor)
        })
    }
}
