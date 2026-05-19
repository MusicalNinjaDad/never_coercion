#![allow(unused_variables)]
#![feature(never_type)]

/// A wrapper around a result which is always OK. We rely on X being a ZST for
/// various optimisations and functionality. (See SAFETY, below, for details on the
/// consequences.)
///
/// ## SAFETY
/// - X must be a zero-sized type. We have no way to ensure that the compiler
///   will validate this, so the constructor and .map_err() are both `unsafe`.  
pub struct Always<T, X>(Result<T, X>);

impl<T, X> Always<T, X> {
    /// ## SAFETY
    /// - X must be a zero-sized type. When calling `new` you must guarantee that
    ///   this is the case.
    pub unsafe fn new(t: T) -> Self {
        Self(Result::Ok(t))
    }

    pub fn map<F, U>(self, f: F) -> Always<U, X>
    where
        F: FnOnce(T) -> U,
    {
        Always(self.0.map(f))
    }

    /// Use map_err to change e.g. Always<String, !> to Always<String, Infallible>
    ///
    /// ## SAFETY
    /// - Z must be a zero-sized type. When calling `map_err` you must guarantee that
    ///   this is the case.
    pub unsafe fn map_err<Z>(self) -> Always<T, Z> {
        unsafe { Always(Ok(self.0.unwrap_unchecked())) }
    }
}

pub enum ZST {}

fn main() {
    let bang: Always<u32, !> = unsafe { Always::new(5) };
    let bang = bang.map(|x| x+1);
    let custom_zst: Always<u32, ZST> = unsafe { bang.map_err() };
}
