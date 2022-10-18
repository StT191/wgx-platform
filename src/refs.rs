// #![feature(box_into_inner)]

use std::{ops::Deref, borrow::Borrow};


// smart pointer to an immutable static value that allows thread-safe sharing without reference counting
// the value will never be dropped
pub struct Static<T> {
    ptr: *const T,
}

impl<T> Static<T> {
    pub fn new(val: T) -> Self {
        Self { ptr: Box::into_raw(Box::new(val)) }
    }
}

impl<T> From<T> for Static<T> {
    fn from(val: T) -> Self {
        Self::new(val)
    }
}

impl<T> Deref for Static<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.ptr }
    }
}

impl<T> AsRef<T> for Static<T> {
    fn as_ref(&self) -> &T {
        unsafe { &*self.ptr }
    }
}

impl<T> Borrow<T> for Static<T> {
    fn borrow(&self) -> &T {
        unsafe { &*self.ptr }
    }
}

impl<T> Clone for Static<T> {
    fn clone(&self) -> Self {
        Self { ptr: self.ptr }
    }
}

impl<T> Copy for Static<T> {}


unsafe impl<T: Send> Send for Static<T> {}
unsafe impl<T: Sync> Sync for Static<T> {}