#![no_std]

pub mod driver;
pub mod hc138;
pub mod mutex;

use embedded_hal::digital::Error as HalError;

/// A trait for demultiplexers that provide multiple "Y" outputs,
/// each of which can be driven active or inactive.
pub trait Demultiplexer {
    type Error: HalError;

    type Parts<'a>
    where
        Self: 'a;

    fn split_demux(&mut self) -> Self::Parts<'_>;
}
