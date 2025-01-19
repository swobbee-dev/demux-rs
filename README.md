# demux-rs

> A Rust library (`no_std`) providing a generic **Demultiplexer** trait, plus an implementation for the **74HC138** demultiplexer.

## Overview

- **`Demultiplexer` trait**: a generic interface for splitting a device into multiple output pins  
- **`74HC138` driver**: a high-level Rust API for controlling the 74HC138 chip via [`embedded-hal`](https://github.com/rust-embedded/embedded-hal)  
- **No-std compatible**: works on embedded systems without an operating system  

When you **split** a `74HC138`, you get 8 output pins (`Y0`..`Y7`), each conforming to [`embedded-hal`’s `OutputPin`](https://docs.rs/embedded-hal/latest/embedded_hal/digital/trait.OutputPin.html) trait. You can then activate or deactivate each output individually.

Example Usage:

```
use demux_rs::hc138::HC138;
use embedded_hal::digital::OutputPin;

// Suppose you have four microcontroller pins implementing `OutputPin`:
let pin_a0 = /* ... */;
let pin_a1 = /* ... */;
let pin_a2 = /* ... */;
let pin_g1 = /* ... */;

// Create the 74HC138 driver
let mut demux = HC138::new(pin_a0, pin_a1, pin_a2, pin_g1);

// Split into 8 outputs (Y0..Y7)
let mut parts = demux.split();

// Each output (e.g. y0) is an `OutputPin`:
parts.y0.set_low().unwrap();  // Activates Y0 (low)
parts.y0.set_high().unwrap(); // Deactivates Y0 (high)
parts.y1.set_low().unwrap();  // Activates Y1, etc.
````

Licensed under either of

    MIT License
    Apache License, Version 2.0

at your option.