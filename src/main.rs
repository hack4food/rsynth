//! Play a sine wave for several seconds.
//!
//! A rusty adaptation of the official PortAudio C "paex_sine.c" example by Phil Burk and Ross
//! Bencina.

extern crate portaudio;

use portaudio as pa;
use std::f64::consts::PI;

const CHANNELS: i32 = 2;
const NUM_SECONDS: i32 = 5;
const SAMPLE_RATE: f64 = 44_100.0;
const FRAMES_PER_BUFFER: u32 = 64;

fn main() {
    match run() {
        Ok(_) => {},
        e => {
            eprintln!("Example failed with the following: {:?}", e);
        }
    }
}

struct Wavetable {
    data: Vec<f32>,
    right_phase: u32,
    left_phase: u32,
}

impl Wavetable {
    fn new(table_size: usize) -> Wavetable {
        let mut sine = Vec::with_capacity(table_size);
        for i in 0..table_size {
            sine.push((i as f64 / table_size as f64 * PI * 2.0).sin() as f32);
        }
        Wavetable {
            data: sine,
            right_phase: 0,
            left_phase: 0,
        }
    }

    fn get_next_and_step(&mut self) -> (f32, f32) { // orz, refactor to come
        self.left_phase += 1;
        self.right_phase += 3;

        let table_size = self.data.len() as u32;
        if self.left_phase >= table_size { self.left_phase -= table_size; }
        if self.right_phase >= table_size { self.right_phase -= table_size; }

        (self.data[self.left_phase as usize], self.data[self.right_phase as usize])
    }

    fn get_next(&self, pos: usize) -> (f32, f32) { // orz, refactor to come
        let left_phase = (1 * pos) % self.data.len();
        let right_phase = (3 * pos) % self.data.len();
        (self.data[left_phase], self.data[right_phase])
    }
}

fn run() -> Result<(), pa::Error> {

    println!("PortAudio Test: output sine wave. SR = {}, BufSize = {}", SAMPLE_RATE, FRAMES_PER_BUFFER);

    // Initialise sinusoidal wavetable.
    let mut wavetable = Wavetable::new(200);

    let pa = pa::PortAudio::new()?;

    let mut settings = pa.default_output_stream_settings(CHANNELS, SAMPLE_RATE, FRAMES_PER_BUFFER)?;
    // we won't output out of range samples so don't bother clipping them.
    settings.flags = pa::stream_flags::CLIP_OFF;

    // This routine will be called by the PortAudio engine when audio is needed. It may called at
    // interrupt level on some machines so don't do anything that could mess up the system like
    // dynamic resource allocation or IO.
    let callback = move |pa::OutputStreamCallbackArgs { buffer, frames, .. }| {
        // TODO: implement logic in order to handle wrapping around the table properly
        let mut idx = 0;
        for frame in 0..frames {
            let (left, right) = /*wavetable.get_next(frame); */ wavetable.get_next_and_step();
            buffer[idx]   = left;
            buffer[idx+1] = right;

            idx += 2;
        }
        pa::Continue
    };

    let mut stream = pa.open_non_blocking_stream(settings, callback)?;

    stream.start()?;

    println!("Play for {} seconds.", NUM_SECONDS);
    pa.sleep(NUM_SECONDS * 1_000);

    stream.stop()?;
    stream.close()?;

    println!("Test finished.");

    Ok(())
}
