use core::cell::RefCell;

use embedded_hal::digital::{Error as HalError, ErrorType, OutputPin};

use crate::driver::{HC138Driver, HC138Error};
use crate::mutex::PortMutex;

/// A trait for demultiplexers that provide multiple "Y" outputs,
/// each of which can be driven active or inactive.
pub trait Demultiplexer {
    type Error: HalError;

    /// Type containing the parted-out pins (Y0..Y7).
    type Parts<'a>
    where
        Self: 'a;

    /// Splits the demultiplexer into its 8 output pins.
    fn split_demux(&mut self) -> Self::Parts<'_>;
}

/// High-level 74HC138 wrapper that can be backed by any `PortMutex`.
///
/// - `M` is the mutex type, e.g. `RefCell<HC138Driver<...>>`.
/// - `A0`, `A1`, `A2`, `G1` are pin types implementing `OutputPin`.
pub struct HC138<M, A0, A1, A2, G1>
where
    M: PortMutex<Port = HC138Driver<A0, A1, A2, G1>>,
    A0: OutputPin,
    A1: OutputPin,
    A2: OutputPin,
    G1: OutputPin,
{
    pub(crate) driver: M,
}

// ----------------------------------------------------------------------------
// 1) A simpler "new()" that *always* returns a RefCell-based HC138
// ----------------------------------------------------------------------------

impl<A0, A1, A2, G1> HC138<RefCell<HC138Driver<A0, A1, A2, G1>>, A0, A1, A2, G1>
where
    A0: OutputPin,
    A1: OutputPin,
    A2: OutputPin,
    G1: OutputPin,
{
    /// Single-threaded constructor: always uses a `RefCell<HC138Driver<...>>`.
    ///
    /// ```no_run
    /// // Example usage:
    /// // let a0 = ...;
    /// // let a1 = ...;
    /// // let a2 = ...;
    /// // let g1 = ...;
    /// // let mut hc138 = HC138::new(a0, a1, a2, g1); // no generics needed!
    /// ```
    pub fn new(a0: A0, a1: A1, a2: A2, g1: G1) -> Self {
        let driver = HC138Driver::new(a0, a1, a2, g1)
            .expect("Failed to initialize 74HC138 pins");
        Self {
            driver: RefCell::new(driver),
        }
    }
}

// ----------------------------------------------------------------------------
// 2) A "new_with_mutex()" for more advanced concurrency or customization
// ----------------------------------------------------------------------------

impl<M, A0, A1, A2, G1> HC138<M, A0, A1, A2, G1>
where
    M: PortMutex<Port = HC138Driver<A0, A1, A2, G1>>,
    A0: OutputPin,
    A1: OutputPin,
    A2: OutputPin,
    G1: OutputPin,
{
    /// Fully generic constructor that accepts a user-supplied `PortMutex`.
    ///
    /// Call this if you need concurrency locking beyond `RefCell`.
    ///
    /// ```no_run
    /// # use your_crate::hc138::HC138;
    /// # use your_crate::driver::HC138Driver;
    /// # use your_crate::shared::PortMutex;
    /// # use embedded_hal::digital::OutputPin;
    /// struct SomeMutex<T>(T);
    /// impl<T> PortMutex for SomeMutex<T> {
    ///     type Port = T;
    ///     fn create(port: Self::Port) -> Self { SomeMutex(port) }
    ///     fn lock<R, F: FnOnce(&mut Self::Port) -> R>(&self, f: F) -> R { unimplemented!() }
    /// }
    ///
    /// # struct PinA0; impl OutputPin for PinA0 { fn set_low(&mut self)->Result<(),()> {Ok(())} fn set_high(&mut self)->Result<(),()> {Ok(())}}
    /// # struct PinA1; impl OutputPin for PinA1 { fn set_low(&mut self)->Result<(),()> {Ok(())} fn set_high(&mut self)->Result<(),()> {Ok(())}}
    /// # struct PinA2; impl OutputPin for PinA2 { fn set_low(&mut self)->Result<(),()> {Ok(())} fn set_high(&mut self)->Result<(),()> {Ok(())}}
    /// # struct PinG1; impl OutputPin for PinG1 { fn set_low(&mut self)->Result<(),()> {Ok(())} fn set_high(&mut self)->Result<(),()> {Ok(())}}
    ///
    /// // Usage:
    /// fn example(a0: PinA0, a1: PinA1, a2: PinA2, g1: PinG1) {
    ///     let hc138 = HC138::new_with_mutex(a0, a1, a2, g1, SomeMutex::create);
    ///     // ...
    /// }
    /// ```
    pub fn new_with_mutex(
        a0: A0,
        a1: A1,
        a2: A2,
        g1: G1,
        make_mutex: impl FnOnce(HC138Driver<A0, A1, A2, G1>) -> M,
    ) -> Self {
        let driver = HC138Driver::new(a0, a1, a2, g1)
            .expect("Failed to initialize 74HC138 pins");
        Self {
            driver: make_mutex(driver),
        }
    }

    /// Split into eight output pins (Y0..Y7).
    pub fn split(&mut self) -> Parts<'_, M, A0, A1, A2, G1> {
        Parts {
            y0: YxPin::new(&self.driver, 0),
            y1: YxPin::new(&self.driver, 1),
            y2: YxPin::new(&self.driver, 2),
            y3: YxPin::new(&self.driver, 3),
            y4: YxPin::new(&self.driver, 4),
            y5: YxPin::new(&self.driver, 5),
            y6: YxPin::new(&self.driver, 6),
            y7: YxPin::new(&self.driver, 7),
        }
    }
}

// We also implement Demultiplexer for all versions of HC138<M, ...>
impl<M, A0, A1, A2, G1> Demultiplexer for HC138<M, A0, A1, A2, G1>
where
    M: PortMutex<Port = HC138Driver<A0, A1, A2, G1>>,
    A0: OutputPin,
    A1: OutputPin,
    A2: OutputPin,
    G1: OutputPin,
{
    type Error = HC138Error;
    type Parts<'a> = Parts<'a, M, A0, A1, A2, G1> where Self: 'a;

    fn split_demux(&mut self) -> Self::Parts<'_> {
        self.split()
    }
}

/// Holds the 8 Yx pins after splitting.
pub struct Parts<'a, M, A0, A1, A2, G1>
where
    M: PortMutex<Port = HC138Driver<A0, A1, A2, G1>> + 'a,
    A0: OutputPin + 'a,
    A1: OutputPin + 'a,
    A2: OutputPin + 'a,
    G1: OutputPin + 'a,
{
    pub y0: YxPin<'a, M, A0, A1, A2, G1>,
    pub y1: YxPin<'a, M, A0, A1, A2, G1>,
    pub y2: YxPin<'a, M, A0, A1, A2, G1>,
    pub y3: YxPin<'a, M, A0, A1, A2, G1>,
    pub y4: YxPin<'a, M, A0, A1, A2, G1>,
    pub y5: YxPin<'a, M, A0, A1, A2, G1>,
    pub y6: YxPin<'a, M, A0, A1, A2, G1>,
    pub y7: YxPin<'a, M, A0, A1, A2, G1>,
}

/// A proxy implementing `embedded_hal::digital::OutputPin` for one Y output.
pub struct YxPin<'a, M, A0, A1, A2, G1>
where
    M: PortMutex<Port = HC138Driver<A0, A1, A2, G1>> + 'a,
    A0: OutputPin + 'a,
    A1: OutputPin + 'a,
    A2: OutputPin + 'a,
    G1: OutputPin + 'a,
{
    driver: &'a M,
    channel: u8,
}

impl<'a, M, A0, A1, A2, G1> YxPin<'a, M, A0, A1, A2, G1>
where
    M: PortMutex<Port = HC138Driver<A0, A1, A2, G1>>,
    A0: OutputPin,
    A1: OutputPin,
    A2: OutputPin,
    G1: OutputPin,
{
    pub(crate) fn new(driver: &'a M, channel: u8) -> Self {
        Self { driver, channel }
    }
}

impl<'a, M, A0, A1, A2, G1> ErrorType for YxPin<'a, M, A0, A1, A2, G1>
where
    M: PortMutex<Port = HC138Driver<A0, A1, A2, G1>>,
    A0: OutputPin,
    A1: OutputPin,
    A2: OutputPin,
    G1: OutputPin,
{
    type Error = HC138Error;
}

impl<'a, M, A0, A1, A2, G1> OutputPin for YxPin<'a, M, A0, A1, A2, G1>
where
    M: PortMutex<Port = HC138Driver<A0, A1, A2, G1>>,
    A0: OutputPin,
    A1: OutputPin,
    A2: OutputPin,
    G1: OutputPin,
{
    fn set_low(&mut self) -> Result<(), Self::Error> {
        self.driver.lock(|drv| drv.set_low(self.channel))
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        self.driver.lock(|drv| drv.set_high(self.channel))
    }
}

// Test-specific helper for the RefCell-based version
#[cfg(test)]
impl<A0, A1, A2, G1> HC138<RefCell<HC138Driver<A0, A1, A2, G1>>, A0, A1, A2, G1>
where
    A0: embedded_hal::digital::OutputPin,
    A1: embedded_hal::digital::OutputPin,
    A2: embedded_hal::digital::OutputPin,
    G1: embedded_hal::digital::OutputPin,
{
    /// Consumes self and returns the underlying mock pins so that `.done()` can be called.
    /// Only available in tests.
    pub fn test_release(self) -> (A0, A1, A2, G1) {
        self.driver.into_inner().release()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::driver::HC138Error;
    use embedded_hal_mock::eh1::digital::{Mock, State, Transaction};

    #[test]
    fn test_err() {
        let expectations_a0 = [
            Transaction::set(State::Low),  // new() init
            Transaction::set(State::Low),  // set_low(0) => bit0=0
            Transaction::set(State::High), // set_low(1) => bit0=1
        ];
        let mock_a0 = Mock::new(&expectations_a0);

        let expectations_a1 = [
            Transaction::set(State::Low),
            Transaction::set(State::Low),
            Transaction::set(State::Low),
        ];
        let mock_a1 = Mock::new(&expectations_a1);

        let expectations_a2 = [
            Transaction::set(State::Low),
            Transaction::set(State::Low),
            Transaction::set(State::Low),
        ];
        let mock_a2 = Mock::new(&expectations_a2);

        let expectations_g1 = [
            Transaction::set(State::High),
            Transaction::set(State::Low), // enable channel 0
            // channel 1 attempt => AlreadySelected => no calls
            Transaction::set(State::High), // disable channel 0
            Transaction::set(State::Low),  // enable channel 1
        ];
        let mock_g1 = Mock::new(&expectations_g1);

        // Just use the single-threaded constructor:
        let mut dev = HC138::new(mock_a0, mock_a1, mock_a2, mock_g1);
        let parts = dev.split();

        let mut y0 = parts.y0;
        let mut y1 = parts.y1;

        y0.set_low().unwrap();

        // Attempt to select Y1 => AlreadySelected => no pin calls
        let err = y1.set_low();
        assert_eq!(err, Err(HC138Error::AlreadySelected));

        y0.set_high().unwrap();

        // no error this time
        y1.set_low().unwrap();

        let (mut a0, mut a1, mut a2, mut g1) = dev.test_release();
        a0.done();
        a1.done();
        a2.done();
        g1.done();
    }
}
