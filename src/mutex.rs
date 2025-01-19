//! A generic “mutex” trait and a simple `RefCell`-based implementation.

use core::cell::RefCell;

/// Common interface for mutex-like wrappers.
pub trait PortMutex {
    type Port;

    fn create(port: Self::Port) -> Self;

    fn lock<R, F: FnOnce(&mut Self::Port) -> R>(&self, f: F) -> R;
}

/// A simple single-threaded “mutex” using `RefCell`.
///
/// Suitable for many embedded-hal use-cases in a single execution context.
impl<T> PortMutex for RefCell<T> {
    type Port = T;

    fn create(port: Self::Port) -> Self {
        RefCell::new(port)
    }

    fn lock<R, F: FnOnce(&mut Self::Port) -> R>(&self, f: F) -> R {
        let mut borrowed = self.borrow_mut();
        f(&mut borrowed)
    }
}
