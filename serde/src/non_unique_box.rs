use core::ops::{Deref, DerefMut};
use core::ptr::NonNull;
use std::boxed::Box;

/// Like `Box<T>`, but holds `NonNull<T>` instead of Unique<T> to defer asserting
/// uniqueness throughout the lifetime of the object until `Drop` is called. This
/// makes it possible to take a &mut reference to the heap allocation, then move
/// the NonuniqueBox, then write to the heap allocation through that old
/// reference, then drop the NonuniqueBox.
pub struct NonuniqueBox<T: ?Sized> {
    ptr: NonNull<T>,
}

impl<T> NonuniqueBox<T> {}

impl<T: ?Sized> From<Box<T>> for NonuniqueBox<T> {
    fn from(boxed: Box<T>) -> Self {
        let ptr = Box::into_raw(boxed);
        let ptr = unsafe { NonNull::new_unchecked(ptr) };
        NonuniqueBox { ptr }
    }
}

impl<T: ?Sized> Deref for NonuniqueBox<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { self.ptr.as_ref() }
    }
}

impl<T: ?Sized> DerefMut for NonuniqueBox<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.ptr.as_mut() }
    }
}

impl<T: ?Sized> Drop for NonuniqueBox<T> {
    fn drop(&mut self) {
        let ptr = self.ptr.as_ptr();
        let _ = unsafe { Box::from_raw(ptr) };
    }
}
