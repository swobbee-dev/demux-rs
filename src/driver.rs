//! Low-level driver that directly owns the 74HC138 pins and tracks state.

use embedded_hal::digital::{Error as HalError, ErrorKind, OutputPin};

/// Possible errors from the 74HC138 driver.
#[derive(Debug, PartialEq, Eq)]
pub enum HC138Error {
    /// Attempted to select a different channel when one is already active.
    AlreadySelected,
    /// Underlying pin error from the HAL pin.
    PinError,
}

impl HalError for HC138Error {
    fn kind(&self) -> ErrorKind {
        ErrorKind::Other
    }
}

/// The low-level driver that manages A0, A1, A2, and G1 pins directly.
pub struct HC138Driver<A0, A1, A2, G1>
where
    A0: OutputPin,
    A1: OutputPin,
    A2: OutputPin,
    G1: OutputPin,
{
    pub(crate) a0: A0,
    pub(crate) a1: A1,
    pub(crate) a2: A2,
    pub(crate) g1: G1,
    pub(crate) current_selected: Option<u8>,
}

impl<A0, A1, A2, G1> HC138Driver<A0, A1, A2, G1>
where
    A0: OutputPin,
    A1: OutputPin,
    A2: OutputPin,
    G1: OutputPin,
{
    /// Create a new driver and set all outputs high (inactive).
    pub fn new(mut a0: A0, mut a1: A1, mut a2: A2, mut g1: G1) -> Result<Self, HC138Error> {
        // On reset: all outputs high => G1 = high => device disabled
        a0.set_low().map_err(|_| HC138Error::PinError)?;
        a1.set_low().map_err(|_| HC138Error::PinError)?;
        a2.set_low().map_err(|_| HC138Error::PinError)?;
        g1.set_high().map_err(|_| HC138Error::PinError)?;

        Ok(Self {
            a0,
            a1,
            a2,
            g1,
            current_selected: None,
        })
    }

    /// Select (drive low) the specified channel (0..7).
    pub fn set_low(&mut self, channel: u8) -> Result<(), HC138Error> {
        if let Some(current) = self.current_selected {
            if current != channel {
                return Err(HC138Error::AlreadySelected);
            }
            // same channel => already low, no-op
            return Ok(());
        }

        self.set_address_bits(channel)?;
        self.g1.set_low().map_err(|_| HC138Error::PinError)?;
        self.current_selected = Some(channel);
        Ok(())
    }

    /// De-select (drive high) this channel if it was active.
    pub fn set_high(&mut self, channel: u8) -> Result<(), HC138Error> {
        if let Some(current) = self.current_selected {
            if current == channel {
                self.g1.set_high().map_err(|_| HC138Error::PinError)?;
                self.current_selected = None;
            }
        }
        Ok(())
    }

    fn set_address_bits(&mut self, channel: u8) -> Result<(), HC138Error> {
        let bit0 = (channel & 0b001) != 0;
        let bit1 = (channel & 0b010) != 0;
        let bit2 = (channel & 0b100) != 0;

        if bit0 {
            self.a0.set_high().map_err(|_| HC138Error::PinError)?;
        } else {
            self.a0.set_low().map_err(|_| HC138Error::PinError)?;
        }

        if bit1 {
            self.a1.set_high().map_err(|_| HC138Error::PinError)?;
        } else {
            self.a1.set_low().map_err(|_| HC138Error::PinError)?;
        }

        if bit2 {
            self.a2.set_high().map_err(|_| HC138Error::PinError)?;
        } else {
            self.a2.set_low().map_err(|_| HC138Error::PinError)?;
        }

        Ok(())
    }

    #[cfg(test)]
    /// For testing only: release the pins so we can call `.done()` on mocks.
    pub fn release(self) -> (A0, A1, A2, G1) {
        (self.a0, self.a1, self.a2, self.g1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use embedded_hal_mock::eh1::digital::{Mock, State, Transaction};

    #[test]
    fn test_driver_init_and_select() {
        // 1) new() => a0=low, a1=low, a2=low, g1=high
        // 2) set_low(0) => a0=low, a1=low, a2=low, g1=low
        // 3) set_high(0) => g1=high

        let expectations_a0 = [
            Transaction::set(State::Low), // init
            Transaction::set(State::Low), // set_address_bits(0)
        ];
        let mock_a0 = Mock::new(&expectations_a0);

        let expectations_a1 = [Transaction::set(State::Low), Transaction::set(State::Low)];
        let mock_a1 = Mock::new(&expectations_a1);

        let expectations_a2 = [Transaction::set(State::Low), Transaction::set(State::Low)];
        let mock_a2 = Mock::new(&expectations_a2);

        let expectations_g1 = [
            Transaction::set(State::High),
            Transaction::set(State::Low),
            Transaction::set(State::High),
        ];
        let mock_g1 = Mock::new(&expectations_g1);

        let mut drv =
            HC138Driver::new(mock_a0, mock_a1, mock_a2, mock_g1).expect("Failed to create driver");

        drv.set_low(0).unwrap();
        drv.set_high(0).unwrap();

        let (mut a0, mut a1, mut a2, mut g1) = drv.release();
        a0.done();
        a1.done();
        a2.done();
        g1.done();
    }

    #[test]
    fn test_select_already_selected() {
        // new() => a0=low, a1=low, a2=low, g1=high
        // set_low(0) => a0=low, a1=low, a2=low, g1=low
        // set_low(1) => AlreadySelected => no calls

        let mock_a0 = Mock::new(&[Transaction::set(State::Low), Transaction::set(State::Low)]);
        let mock_a1 = Mock::new(&[Transaction::set(State::Low), Transaction::set(State::Low)]);
        let mock_a2 = Mock::new(&[Transaction::set(State::Low), Transaction::set(State::Low)]);
        let mock_g1 = Mock::new(&[Transaction::set(State::High), Transaction::set(State::Low)]);

        let mut drv = HC138Driver::new(mock_a0, mock_a1, mock_a2, mock_g1).unwrap();

        drv.set_low(0).unwrap();
        let err = drv.set_low(1).unwrap_err();
        assert_eq!(err, HC138Error::AlreadySelected);

        let (mut a0, mut a1, mut a2, mut g1) = drv.release();
        a0.done();
        a1.done();
        a2.done();
        g1.done();
    }
}
