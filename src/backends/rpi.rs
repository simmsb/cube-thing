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
        for (idx, layer) in frame.layers().iter().rev().enumerate() {
            self.rclk.set_low();

            let height = 1u8 << idx;
            self.out_byte(height);

            for row in layer {
                for brightness in row {
                    let bit = self.ticker.compute_pwm(*brightness);
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
}

impl PWMTicker {
    pub fn new() -> Self {
        Self { frame: 0 }
    }

    pub fn compute_pwm(&self, brightness: u8) -> bool {
        const BRIGHTNESS_PATS: [u32; 32] = [
            0b00000000000000000000000000000000,
            0b00000000000000000000000000000001,
            0b00000000000000100000000000000001,
            0b00000001000000000000010000000001,
            0b00000010000000010000000100000001,
            0b00001000000010000000100000100001,
            0b00100000100000100000100000100001,
            0b00100001000010000010000100010001,
            0b10000100010000100010001000100001,
            0b10001000100010001000100010001001,
            0b10010001000100100010001001001001,
            0b10010010010010010010010010010010,
            0b10010100100101001001001010010010, // 12
            0b10010100101001010010100101001010,
            0b10010100101001010010100101001011,
            0b10010101001010100101010010101001,
            0b10010101001010101101010010101001,
            0b10110101001010101101010010101001,
            0b10110101001010101101010110101001,
            0b10110101011010101101010110101001,
            0b10110101011010101101010110101101,
            0b10110101011011101101010110101101,
            0b10110111011011101101010110101101,
            0b10110111011011101101110110101101,
            0b10110111011011101101110111101101,
            0b10110111111011101101110111101101,
            0b10110111111011101101110111111101,
            0b11110111111011101101110111111101,
            0b11110111111011101111110111111101,
            0b11110111111111101111111111111101,
            0b11111111111111101111111111111111,
            0b11111111111111111111111111111111,
        ];

        let pattern = BRIGHTNESS_PATS[(brightness / 8) as usize];

        pattern.view_bits::<Lsb0>()[self.frame as usize]
    }

    pub fn next_frame(&mut self) {
        self.frame += 1;
        self.frame %= 32;
    }
}
