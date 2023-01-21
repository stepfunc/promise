//! Helpers types and functions for making oo-bindgen future interfaces drop-safe.
#![deny(
    dead_code,
    arithmetic_overflow,
    invalid_type_param_default,
    missing_fragment_specifier,
    mutable_transmutes,
    no_mangle_const_items,
    overflowing_literals,
    patterns_in_fns_without_body,
    pub_use_of_private_extern_crate,
    unknown_crate_types,
    order_dependent_trait_objects,
    illegal_floating_point_literal_pattern,
    improper_ctypes,
    late_bound_lifetime_arguments,
    non_camel_case_types,
    non_shorthand_field_patterns,
    non_snake_case,
    non_upper_case_globals,
    no_mangle_generic_items,
    private_in_public,
    stable_features,
    type_alias_bounds,
    tyvar_behind_raw_pointer,
    unconditional_recursion,
    unused_comparisons,
    unreachable_pub,
    anonymous_parameters,
    missing_copy_implementations,
    missing_debug_implementations,
    missing_docs,
    trivial_casts,
    trivial_numeric_casts,
    unused_import_braces,
    unused_qualifications,
    clippy::all
)]
#![forbid(
    unsafe_code,
    rustdoc::broken_intra_doc_links,
    unaligned_references,
    while_true,
    bare_trait_objects
)]

/// Types convertible to a Promise must implement this type
pub trait FutureType<V> {
    /// The value that will be returned if a Promise of this type is dropped without being completed
    fn on_drop() -> V;

    /// Complete the future with the specified value
    fn complete(self, result: V);
}

/// A Promise is a type that is guaranteed to complete its underlying FutureType,
/// even if it is dropped.
#[derive(Debug)]
pub struct Promise<T, V>
where
    T: FutureType<V>,
{
    inner: Option<T>,
    _v: std::marker::PhantomData<V>,
}

impl<T, V> Promise<T, V>
where
    T: FutureType<V>,
{
    /// Construct a promise from a FutureType
    fn new(inner: T) -> Self {
        Self {
            inner: Some(inner),
            _v: Default::default(),
        }
    }

    /// Complete the promise, consuming it
    pub fn complete(mut self, result: V) {
        if let Some(x) = self.inner.take() {
            x.complete(result);
        }
    }
}

/// Wrap a type that implements FutureType into a drop-safe promise
pub fn wrap<T, V>(callback: T) -> Promise<T, V>
where
    T: FutureType<V>,
{
    Promise::new(callback)
}

impl<T, V> Drop for Promise<T, V>
where
    T: FutureType<V>,
{
    fn drop(&mut self) {
        if let Some(cb) = self.inner.take() {
            cb.complete(T::on_drop());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Borrowed<'a> {
        vec: &'a mut Vec<Result<u32, &'static str>>,
    }

    impl<'a> FutureType<Result<u32, &'static str>> for Borrowed<'a> {
        fn on_drop() -> Result<u32, &'static str> {
            Err("dropped")
        }

        fn complete(self, result: Result<u32, &'static str>) {
            self.vec.push(result);
        }
    }

    #[test]
    fn completes_on_drop() {
        let mut output = Vec::new();
        let _ = wrap(Borrowed { vec: &mut output });
        assert_eq!(output.as_slice(), [Err("dropped")]);
    }

    #[test]
    fn only_completes_once_on_success() {
        let mut output = Vec::new();
        let promise = wrap(Borrowed { vec: &mut output });
        promise.complete(Ok(42));
        assert_eq!(output.as_slice(), [Ok(42)]);
    }

    #[test]
    fn only_completes_once_on_failure() {
        let mut output = Vec::new();
        let promise = wrap(Borrowed { vec: &mut output });
        promise.complete(Err("fail"));
        assert_eq!(output.as_slice(), [Err("fail")]);
    }
}
