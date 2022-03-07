use bitvec::{order::Lsb0, view::BitView};
use rppal::gpio::{self, Gpio, Level, OutputPin};
use std::time::{Duration, Instant};

#[derive(Clone, Debug)]
pub struct Frame([[[u8; 8]; 8]; 8]);

impl Frame {
    pub const LAYERS: usize = 8;
    pub const ROWS: usize = 8;
    pub const COLUMNS: usize = 8;

    pub fn new() -> Self {
        Self([[[0u8; 8]; 8]; 8])
    }

    pub fn get(&self, x: usize, y: usize, z: usize) -> u8 {
        self.0[y][x][z]
    }

    pub fn set(&mut self, x: usize, y: usize, z: usize, val: u8) {
        self.0[y][x][z] = val;
    }

    pub fn layer_mut(&mut self, n: usize) -> &mut [[u8; 8]; 8] {
        &mut self.0[n]
    }

    pub fn pixels<'a>(&'a self) -> impl Iterator<Item = (u8, u8, u8, u8)> + 'a {
        self.0.iter().zip(0..8u8).flat_map(|(layer, y)| {
            layer
                .iter()
                .zip(0..8u8)
                .flat_map(move |(row, x)| row.iter().zip(0..8u8).map(move |(pix, z)| (x, z, pix)))
                .map(move |(x, z, pix)| (x, y, z, *pix))
        })
    }

    pub fn pixels_mut<'a>(&'a mut self) -> impl Iterator<Item = (u8, u8, u8, &'a mut u8)> + 'a {
        self.0.iter_mut().zip(0..8u8).flat_map(|(layer, y)| {
            layer
                .iter_mut()
                .zip(0..8u8)
                .flat_map(move |(row, x)| {
                    row.iter_mut().zip(0..8u8).map(move |(pix, z)| (x, z, pix))
                })
                .map(move |(x, z, pix)| (x, y, z, pix))
        })
    }
}

const SERIAL: u8 = 25;
const RCLK: u8 = 24;
const SRCLK: u8 = 23;

//
// I should really just make this generic over backends, but I'm too lazy

pub struct Pins {
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

    fn display_frame(&mut self, frame: &Frame, pwm_ticker: &PWMTicker) {
        for (idx, layer) in frame.0.iter().rev().enumerate() {
            self.rclk.set_low();

            let height = 1u8 << idx;
            self.out_byte(height);

            for row in layer {
                for brightness in row {
                    let bit = pwm_ticker.compute_pwm(*brightness);
                    self.push_pin(bit);
                }
            }

            self.rclk.set_high();
        }
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

pub trait Animation {
    fn next_frame(&mut self, frame: &mut Frame);

    fn ended(&self) -> bool {
        false
    }

    fn reset(&mut self);

    fn with_fps(self, fps: f32) -> FixedFPSAnimation<Self>
    where
        Self: Sized,
    {
        FixedFPSAnimation {
            inner: self,
            interval: Duration::from_secs_f32(1.0 / fps),
            last_frame: Instant::now(),
        }
    }

    fn with_duration(self, duration: Duration) -> TimeLimitedAnimation<Self>
    where
        Self: Sized,
    {
        TimeLimitedAnimation {
            inner: self,
            duration,
            started: Instant::now(),
        }
    }

    fn chain<U: Animation>(self, other: U) -> ChainedAnimation<Self, U>
    where
        Self: Sized,
    {
        ChainedAnimation {
            a: self,
            b: other,
            current: false,
        }
    }
}

pub struct FixedFPSAnimation<T> {
    inner: T,
    interval: Duration,
    last_frame: Instant,
}

impl<T: Animation> Animation for FixedFPSAnimation<T> {
    fn next_frame(&mut self, frame: &mut Frame) {
        let now = Instant::now();
        if now.duration_since(self.last_frame) > self.interval {
            self.inner.next_frame(frame);
            self.last_frame = now;
        }
    }

    fn ended(&self) -> bool {
        self.inner.ended()
    }

    fn reset(&mut self) {
        self.inner.reset();
    }
}

pub struct TimeLimitedAnimation<T> {
    inner: T,
    duration: Duration,
    started: Instant,
}

impl<T: Animation> Animation for TimeLimitedAnimation<T> {
    fn next_frame(&mut self, frame: &mut Frame) {
        self.inner.next_frame(frame);
    }

    fn ended(&self) -> bool {
        if Instant::now().duration_since(self.started) > self.duration {
            return true;
        }

        self.inner.ended()
    }

    fn reset(&mut self) {
        self.inner.reset();
        self.started = Instant::now();
    }
}

pub struct ChainedAnimation<T, U> {
    a: T,
    b: U,
    current: bool,
}

impl<T: Animation, U: Animation> Animation for ChainedAnimation<T, U> {
    fn next_frame(&mut self, frame: &mut Frame) {
        if !self.current {
            if self.a.ended() {
                self.current = !self.current;
                self.b.reset();
            } else {
                self.a.next_frame(frame)
            }
        }

        if self.current && !self.b.ended() {
            self.b.next_frame(frame)
        }
    }

    fn ended(&self) -> bool {
        self.current && self.b.ended()
    }

    fn reset(&mut self) {
        self.a.reset();
        self.current = false;
    }
}

pub struct Driver {
    animation: Box<dyn Animation + Send + Sync>,
    #[cfg(not(feature = "visual"))]
    pins: Pins,
    ticker: PWMTicker,
    frame: Frame,
}

impl Driver {
    pub fn new<T: Animation + Send + Sync + 'static>(animation: T) -> Self {
        #[cfg(not(feature = "visual"))]
        let pins = Pins::new().unwrap();
        let ticker = PWMTicker::new();
        let animation = Box::new(animation) as Box<dyn Animation + Send + Sync>;
        let frame = Frame::new();

        Self {
            animation,
            #[cfg(not(feature = "visual"))]
            pins,
            ticker,
            frame,
        }
    }

    pub fn reset(&mut self) {
        self.animation.reset();
    }

    pub fn step(&mut self) {
        if self.animation.ended() {
            self.animation.reset();
        }

        self.animation.next_frame(&mut self.frame);

        #[cfg(not(feature = "visual"))]
        self.pins.display_frame(&self.frame, &self.ticker);
        self.ticker.next_frame();
    }

    pub fn frame(&self) -> &Frame {
        &self.frame
    }
}
