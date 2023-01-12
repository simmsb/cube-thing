use bitvec::{order::Lsb0, view::BitView};
use rppal::gpio::{self, Gpio, Level, OutputPin};

use crate::frame::Frame;

use super::backend::Backend;

const SERIAL: u8 = 25;
const RCLK: u8 = 24;
const SRCLK: u8 = 23;

pub struct RpiBackend {
    serial: OutputPin,
    rclk: OutputPin,
    srclk: OutputPin,
    ticker: PWMTicker,
    pwm_err: [[[i8; 8]; 8]; 8],
}

impl RpiBackend {
    pub fn new() -> gpio::Result<Self> {
        let gpio = Gpio::new()?;
        let serial = gpio.get(SERIAL)?.into_output();
        let rclk = gpio.get(RCLK)?.into_output();
        let srclk = gpio.get(SRCLK)?.into_output();
        let ticker = PWMTicker::new();

        Ok(RpiBackend {
            serial,
            rclk,
            srclk,
            ticker,
            pwm_err: [[[0; 8]; 8]; 8],
        })
    }

    fn out_byte(&mut self, byte: u8) {
        for bit in byte.view_bits::<Lsb0>() {
            self.push_pin(*bit);
        }
    }

    fn push_pin(&mut self, bit: bool) {
        self.srclk.set_low();
        self.serial
            .write(if bit { Level::High } else { Level::Low });
        self.srclk.set_high();
    }
}

impl Backend for RpiBackend {
    fn display_frame(&mut self, frame: &Frame) {
        for (layer_idx, layer) in frame.layers().iter().rev().enumerate() {
            self.rclk.set_low();

            let height = 1u8 << layer_idx;
            self.out_byte(height);

            for (row_idx, row) in layer.iter().enumerate() {
                for (led_idx, brightness) in row.iter().enumerate() {
                    let err = &mut self.pwm_err[layer_idx][row_idx][led_idx];
                    let bit = self.ticker.compute_pwm(*brightness, err);
                    self.push_pin(bit);
                }
            }

            self.rclk.set_high();
        }

        self.ticker.next_frame();
    }
}

struct PWMTicker {
    frame: u8,
    noise: i32,
}

impl PWMTicker {
    pub fn new() -> Self {
        Self { frame: 0, noise: 1 }
    }

    pub fn compute_pwm(&mut self, brightness: u8, err: &mut i8) -> bool {
        self.noise = (self.noise / 2) ^ -(self.noise % 2) & 0x428e;
        let c = brightness as i32 + *err as i32 + (((self.noise & 0x1) << 1) - 1);
        let output = if c > 15 { 31 } else { 0 };
        *err = (c - output) as i8;

        output == 0
    }

    pub fn next_frame(&mut self) {
        self.frame += 1;
        self.frame %= 32;
    }
}
