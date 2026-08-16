//! `http`, behind the `http` feature.
//!
//! The small types are leaves: a `Uri` is found as itself, and the string it is parsed from is not
//! searched. A `HeaderMap` is walked, offering each name before its value, in the map's own
//! iteration order, which is unspecified.

use std::ops::ControlFlow;

use http::{HeaderMap, HeaderName, HeaderValue, Method, StatusCode, Uri, Version};

use crate::{TypeSift, impl_type_sift_for_leaf};

impl_type_sift_for_leaf!(HeaderName, HeaderValue, Method, StatusCode, Uri, Version);

impl<V: TypeSift> TypeSift for HeaderMap<V> {
    // `T` is taken by the map, so the searched type is `X` here.
    fn visit<'a, X: 'static, B, F>(&'a self, visitor: &mut F) -> ControlFlow<B>
    where
        F: FnMut(&'a X) -> ControlFlow<B>,
    {
        self.visit_self::<X, B, F>(visitor)?;
        self.iter().try_for_each(|(name, value)| {
            name.visit::<X, B, F>(visitor)?;
            value.visit::<X, B, F>(visitor)
        })
    }
}
