//! This shows some of the interrupts that can be generated by UART/Serial.
//! Use a proper serial terminal to connect to the board (espmonitor and
//! espflash won't work)

//% CHIPS: esp32 esp32c2 esp32c3 esp32c6 esp32h2 esp32s2 esp32s3

#![no_std]
#![no_main]

use core::{cell::RefCell, fmt::Write};

use critical_section::Mutex;
use esp_backtrace as _;
use esp_hal::{
    clock::ClockControl,
    delay::Delay,
    gpio,
    peripherals::{Peripherals, UART0},
    prelude::*,
    system::SystemControl,
    uart::{
        config::{AtCmdConfig, Config},
        TxRxPins,
        Uart,
    },
    Blocking,
};

static SERIAL: Mutex<RefCell<Option<Uart<UART0, Blocking>>>> = Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
    let system = SystemControl::new(peripherals.SYSTEM);
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    let delay = Delay::new(&clocks);

    let mut uart0 = Uart::new_with_config(
        peripherals.UART0,
        Config::default(),
        None::<TxRxPins<gpio::NoPinType, gpio::NoPinType>>,
        &clocks,
        Some(interrupt_handler),
    );

    critical_section::with(|cs| {
        uart0.set_at_cmd(AtCmdConfig::new(None, None, None, b'#', None));
        uart0.set_rx_fifo_full_threshold(30).unwrap();
        uart0.listen_at_cmd();
        uart0.listen_rx_fifo_full();

        SERIAL.borrow_ref_mut(cs).replace(uart0);
    });

    loop {
        critical_section::with(|cs| {
            let mut serial = SERIAL.borrow_ref_mut(cs);
            let serial = serial.as_mut().unwrap();
            writeln!(serial, "Hello World! Send a single `#` character or send at least 30 characters and see the interrupts trigger.").ok();
        });

        delay.delay(1.secs());
    }
}

#[handler]
fn interrupt_handler() {
    critical_section::with(|cs| {
        let mut serial = SERIAL.borrow_ref_mut(cs);
        let serial = serial.as_mut().unwrap();

        let mut cnt = 0;
        while let nb::Result::Ok(_c) = serial.read_byte() {
            cnt += 1;
        }
        writeln!(serial, "Read {} bytes", cnt,).ok();

        writeln!(
            serial,
            "Interrupt AT-CMD: {} RX-FIFO-FULL: {}",
            serial.at_cmd_interrupt_set(),
            serial.rx_fifo_full_interrupt_set(),
        )
        .ok();

        serial.reset_at_cmd_interrupt();
        serial.reset_rx_fifo_full_interrupt();
    });
}
