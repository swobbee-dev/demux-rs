//! A generic "mutex" trait and a simple `RefCell`-based implementation.

use core::cell::RefCell;

/// Common interface for mutex-like wrappers.
pub trait PortMutex {
    type Port;

    fn create(port: Self::Port) -> Self;

    fn lock<R, F: FnOnce(&mut Self::Port) -> R>(&self, f: F) -> R;
}

/// A simple single-threaded "mutex" using `RefCell`.
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

/// Implementation for critical-section::Mutex<RefCell<T>>
///
/// This is suitable for multi-threaded or interrupt contexts where critical sections are needed.
#[cfg(feature = "critical-section")]
impl<T> PortMutex for critical_section::Mutex<RefCell<T>> {
    type Port = T;

    fn create(port: Self::Port) -> Self {
        critical_section::Mutex::new(RefCell::new(port))
    }

    fn lock<R, F: FnOnce(&mut Self::Port) -> R>(&self, f: F) -> R {
        critical_section::with(|cs| {
            let mut borrowed = self.borrow_ref_mut(cs);
            f(&mut borrowed)
        })
    }
}
