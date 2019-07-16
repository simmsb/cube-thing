use bitvec::{
    bits::{Bits as _, BitsMut as _},
    cursor,
};
use itertools::Itertools;
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

    fn next_frame(&mut self, now: Instant) -> Option<&mut std::iter::Cycle<II::IntoIter>> {
        let (current_it, duration) = self.iter.next()?;
        let current_it = current_it.into_iter().cycle();

        self.current = Some((current_it, duration));
        self.last_frame_at = now;

        Some(&mut self.current.as_mut().unwrap().0)
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
                self.next_frame(now)?.next()
            } else {
                current_it.next()
            }
        } else {
            self.next_frame(Instant::now())?.next()
        }
    }
}

fn get_bits_should_be_set(brightness: u8) -> u32 {
    // yes, I just gave up and eyeballed it...
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

    BRIGHTNESS_PATS[(brightness / 8) as usize]
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

    let mut raw_frames = [[[[0; 8]; 8]; 8]; 45];

    for frame in 0..45 {
        for layer in 0..8 {
            let idx = ((((frame + layer) as f64 * 8.0).to_radians().sin() + 1.0) * 127.0) as u8;

            raw_frames[frame][layer] = [[idx; 8]; 8];
        }
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
