//! Displays an animated Nyan cat
#![no_std]
#![no_main]
#![feature(generic_const_exprs)]

use bsp::entry;
// use defmt::*;
// use defmt_rtt as _;
use embedded_graphics::geometry::Point;
use embedded_graphics::image::Image;
use embedded_graphics::prelude::*;
use panic_halt as _;
use tinyqoi::Qoi;

use bsp::hal::pio::PIOExt;
use bsp::hal::{
    clocks::{init_clocks_and_plls, Clock},
    pac,
    sio::Sio,
    watchdog::Watchdog,
};
use hub75_pio;
use hub75_pio::dma::DMAExt;

use rp_pico as bsp;

static mut DISPLAY_BUFFER: hub75_pio::DisplayMemory<64, 64, 8> = hub75_pio::DisplayMemory::new();

const FRAMES: [&[u8]; 12] = [
    include_bytes!("../assets/01.qoi"),
    include_bytes!("../assets/02.qoi"),
    include_bytes!("../assets/03.qoi"),
    include_bytes!("../assets/04.qoi"),
    include_bytes!("../assets/05.qoi"),
    include_bytes!("../assets/06.qoi"),
    include_bytes!("../assets/07.qoi"),
    include_bytes!("../assets/08.qoi"),
    include_bytes!("../assets/09.qoi"),
    include_bytes!("../assets/10.qoi"),
    include_bytes!("../assets/11.qoi"),
    include_bytes!("../assets/12.qoi"),
];

#[entry]
fn main() -> ! {
    // info!("Program start");

    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let sio = Sio::new(pac.SIO);

    // External high-speed crystal on the pico board is 12Mhz
    let external_xtal_freq_hz = 12_000_000u32;
    let clocks = init_clocks_and_plls(
        external_xtal_freq_hz,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

    let pins = bsp::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    // Split PIO0 SM
    let (mut pio, sm0, sm1, sm2, _) = pac.PIO0.split(&mut pac.RESETS);

    // Reset DMA
    let resets = pac.RESETS;
    resets.reset.modify(|_, w| w.dma().set_bit());
    resets.reset.modify(|_, w| w.dma().clear_bit());
    while resets.reset_done.read().dma().bit_is_clear() {}

    // Split DMA
    let dma = pac.DMA.split();

    let mut display = unsafe {
        hub75_pio::Display::new(
            &mut DISPLAY_BUFFER,
            hub75_pio::DisplayPins {
                r1: pins.gpio0.into(),
                g1: pins.gpio1.into(),
                b1: pins.gpio2.into(),
                r2: pins.gpio3.into(),
                g2: pins.gpio4.into(),
                b2: pins.gpio5.into(),
                addra: pins.gpio6.into(),
                addrb: pins.gpio7.into(),
                addrc: pins.gpio8.into(),
                addrd: pins.gpio9.into(),
                addre: pins.gpio10.into(),
                clk: pins.gpio11.into(),
                lat: pins.gpio12.into(),
                oe: pins.gpio13.into(),
            },
            &mut pio,
            (sm0, sm1, sm2),
            (dma.ch0, dma.ch1, dma.ch2, dma.ch3),
        )
    };

    // Display Nyancat
    loop {
        for raw_frame in FRAMES {
            let frame = Qoi::new(raw_frame).unwrap();
            Image::new(&frame, Point::zero())
                .draw(&mut display)
                .unwrap();
            Image::new(&frame, Point::zero())
                .translate(Point::new(0, 32))
                .draw(&mut display)
                .unwrap();


            display.commit();
            delay.delay_ms(100);
        }
    }
}
