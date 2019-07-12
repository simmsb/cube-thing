#![feature(const_fn)]

use bitvec::{
    bits::{Bits as _, BitsMut as _},
    cursor,
};
use itertools::{iproduct, Itertools};
use rppal::gpio::{self, Gpio, Level, OutputPin};
use std::time::{Duration, Instant};

type Frame = [Layer; 8];
type Layer = [u8; 8];

const SERIAL: u8 = 25;
const RCLK: u8 = 24;
const SRCLK: u8 = 23;

struct Pins {
    serial: OutputPin,
    rclk: OutputPin,
    srclk: OutputPin,
}

impl Pins {
    fn new() -> gpio::Result<Self> {
        let gpio = Gpio::new()?;
        let serial = gpio.get(SERIAL)?.into_output();
        let rclk = gpio.get(RCLK)?.into_output();
        let srclk = gpio.get(SRCLK)?.into_output();

        Ok(Pins {
            serial,
            rclk,
            srclk,
        })
    }

    fn display_frame(&mut self, frame: &Frame) {
        for (idx, layer) in frame.iter().rev().enumerate() {
            self.display_layer(idx as u8, *layer);
        }
    }

    fn display_layer(&mut self, idx: u8, layer: Layer) {
        self.rclk.set_low();

        let height = 1u8 << idx;
        self.out_byte(height);

        for layer_data in &layer {
            self.out_byte(*layer_data);
        }

        self.rclk.set_high();
    }

    fn out_byte(&mut self, byte: u8) {
        for bit in byte.as_bitslice::<cursor::LittleEndian>() {
            self.push_pin(bit);
        }
    }

    fn push_pin(&mut self, bit: bool) {
        self.srclk.set_low();
        self.serial
            .write(if bit { Level::High } else { Level::Low });
        self.srclk.set_high();
    }
}

struct FrameIter<I, II>
where
    I: Iterator<Item = (II, Duration)>,
    II: IntoIterator,
{
    iter: I,
    current: Option<(std::iter::Cycle<II::IntoIter>, Duration)>,
    last_frame_at: Instant,
}

impl<'a, I, II> FrameIter<I, II>
where
    I: Iterator<Item = (II, Duration)>,
    II: IntoIterator<Item = &'a Frame>,
    II::IntoIter: Clone,
{
    fn new(it: I) -> Self {
        FrameIter {
            iter: it,
            current: None,
            last_frame_at: Instant::now(),
        }
    }
}

impl<'a, I, II> Iterator for FrameIter<I, II>
where
    I: Iterator<Item = (II, Duration)>,
    II: IntoIterator<Item = &'a Frame>,
    II::IntoIter: Clone,
{
    type Item = &'a Frame;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some((ref mut current_it, duration)) = self.current {
            let now = Instant::now();

            if now.duration_since(self.last_frame_at) > duration {
                let (current_it, duration) = self.iter.next()?;
                let mut current_it = current_it.into_iter().cycle();

                let next = current_it.next();

                self.current = Some((current_it, duration));
                self.last_frame_at = now;

                return next;
            } else {
                return current_it.next();
            }
        } else {
            let (current_it, duration) = self.iter.next()?;
            let mut current_it = current_it.into_iter().cycle();

            let next = current_it.next();

            self.current = Some((current_it, duration));
            self.last_frame_at = Instant::now();

            return next;
        }
    }
}

fn get_bits_should_be_set(brightness: u8) -> u32 {
    const BRIGHTNESS_PATS: [u32; 16] = [
        0x00000000, 0x80000000, 0x80008000, 0x80020000, 0x80040000, 0x80100200, 0x80200800,
        0x80402000, 0x80808080, 0x81020400, 0x82082080, 0x84210840, 0x88888888, 0x92492492,
        0xaaaaaaaa, 0xffffffff,
    ];

    BRIGHTNESS_PATS[(brightness / 16) as usize]
}

/// Process faded frames into an array of frames, where each led is toggled
/// dependant on the brightness
fn process_frame(frame: &[[[u8; 8]; 8]; 8]) -> [Frame; 32] {
    let mut result = [[[0; 8]; 8]; 32];

    for (l_idx, layer) in frame.iter().enumerate() {
        for (r_idx, row) in layer.iter().enumerate() {
            for (p_idx, pixel) in row.iter().enumerate() {
                let set_bits = get_bits_should_be_set(*pixel);

                for idx in set_bits
                    .as_bitslice::<cursor::BigEndian>()
                    .iter()
                    .positions(|b| b)
                {
                    result[idx][l_idx][r_idx]
                        .as_mut_bitslice::<cursor::BigEndian>()
                        .set(p_idx, true);
                }
            }
        }
    }

    result
}

fn main() {
    let mut pins = Pins::new().unwrap();

    // let mut raw_frames = [
    //     [
    //         [[0; 8]; 8],
    //         [[32; 8]; 8],
    //         [[64; 8]; 8],
    //         [[96; 8]; 8],
    //         [[128; 8]; 8],
    //         [[160; 8]; 8],
    //         [[192; 8]; 8],
    //         [[224; 8]; 8],
    //     ]; 16];

    // for i in 0..8 {
    //     raw_frames[i].rotate_right(i);
    //     raw_frames[15 - i].rotate_left(i);
    // }

    let mut raw_frames = [[[[0; 8]; 8]; 8]; 90];

    for i in 0..90 {
        let idx = (((i as f64 * 4.0).to_radians().sin() + 1.0) * 127.0) as u8;

        raw_frames[i] = [[[idx; 8]; 8]; 8];
    }

    let frame_datas: Vec<_> = raw_frames
        .into_iter()
        .map(|frame| process_frame(frame))
        .collect();
    let frames: Vec<_> = frame_datas
        .iter()
        .map(|f| (f, Duration::from_millis(50)))
        .collect();

    let frame_it = FrameIter::new(frames.into_iter().cycle());

    println!("starting to render");

    for frame in frame_it {
        pins.display_frame(frame);
    }
}
