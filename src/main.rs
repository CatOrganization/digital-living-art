mod wave;
mod frequency;

extern crate core;

use std::borrow::{Borrow, BorrowMut};
use std::cmp::{min, Ordering};
use std::sync::{Arc, LockResult, Mutex};
use std::thread;
use cpal::{Sample, SampleFormat};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use raylib::prelude::*;

const AUDIO_BUFFER_LEN: usize = 4096*2;

struct AudioDataBuffer {
    raw: [f32; AUDIO_BUFFER_LEN],
    len: usize,
}

impl AudioDataBuffer {
    fn new() -> Self {
        return Self {
            raw: [0.0; AUDIO_BUFFER_LEN],
            len: 0,
        }
    }

    fn push_data(&mut self, data: &[f32]) {
        // If we get more data than we can hold, copy everything we can and move on.
        if data.len() >= AUDIO_BUFFER_LEN {
            self.raw.copy_from_slice(&data[..AUDIO_BUFFER_LEN]);
            self.len = AUDIO_BUFFER_LEN;

            return
        }

        // Gotta move things if we'd go beyond our buffer length
        if data.len() + self.len > AUDIO_BUFFER_LEN {
            self.raw.copy_within(data.len()..self.len, 0);
            self.len = AUDIO_BUFFER_LEN - data.len();
        }

        let end_index = self.len + data.len();
        self.raw[self.len..end_index].copy_from_slice(data);
        self.len = self.len + data.len();
    }

    fn recv_audio_data(&mut self, data: &[f32], _: &cpal::InputCallbackInfo) {

    }

    fn pop_data(&mut self, dst: &mut [f32]) -> usize {
        if self.len == 0 {
            return 0;
        }

        let len_to_copy = min(dst.len(), self.len);

        dst[0..len_to_copy].copy_from_slice(&self.raw[..len_to_copy]);

        if len_to_copy < self.len {
            self.raw.copy_within(len_to_copy..self.len, 0);
            self.len -= len_to_copy;
        } else {
            self.len = 0;
        }

        return len_to_copy;
    }
}

fn main() {
    let buf = Arc::new(Mutex::new(AudioDataBuffer{
        raw: [0.0; AUDIO_BUFFER_LEN],
        len: 0,
    }));

    println!("main thread: {}", thread::current().name().unwrap_or("shit"));

    let audio_host = cpal::default_host();

    println!("{}",cpal::available_hosts().len());

    for device in audio_host.input_devices().expect("no input devices") {
        println!("{}", device.name().expect("device has no name..."))
    }

    let device = audio_host.default_input_device().expect("no audio input device");
    println!("using {}", device.name().expect("no name!!"));

    let err_fn = |err| eprintln!("an error occurred on the output audio stream: {}", err);
    let mut supported_configs_range = device.supported_input_configs()
        .expect("error while querying configs");
    let supported_config = supported_configs_range.next()
        .expect("no supported config?!")
        .with_max_sample_rate();
    let config = supported_config.config();

    println!("{}, {}, {}, {}", match config.buffer_size {
        cpal::BufferSize::Default => "default",
        cpal::BufferSize::Fixed(FrameCount) => {
            "fixed"
        }
    }, config.channels, config.sample_rate.0, match supported_config.sample_format(){
        SampleFormat::I16 => "i16",
        SampleFormat::U16 => "u16",
        SampleFormat::F32 => "f32",
    });

    let default_config = device.default_input_config().expect("no supported config");

    let stream = device.build_input_stream(&config, get_audio_callback(Arc::clone(&buf)), err_fn).expect("idk man");
    stream.play().expect("error playing stream");


    let (mut rl, thread) = raylib::init()
        .size(1440, 900)
        .title("Hello, World")
        .build();

    rl.set_target_fps(60);


    let mut wave_viz = wave::WaveViz::new(1440, 900);
    let mut freq_viz = frequency::FrequencyViz::new(1440, 900);

    // let mut v = freq_viz;
    // let mut v = wave_viz;

    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::DARKGRAY);
        d.draw_fps(12, 24);

        {
            let actual = buf.lock();
            let mut actual = actual.unwrap();
            // d.draw_text(&*format!("copied frames: {}", actual.pop_data(&mut sink[..])), 12, 32, 20, Color::BLACK);
            // d.draw_text(&*format!("max: {}", IntoIterator::into_iter(sink).max_by(|a, b| if a > b { Ordering::Greater } else { Ordering::Less }).unwrap().to_f32()), 12, 56, 20, Color::BLACK);


            // for x in 0..1024 {
            //     let pt = actual.raw[x];
            //     d.draw_circle(x as i32, (pt * 400.0) as i32 + 100, 1.0, Color::RED);
            //     //x += 1;
            // }

            {
                wave_viz.render(d.borrow_mut(), actual.raw);
            }

            {
                freq_viz.render(d.borrow_mut(), actual.raw);
            }
        }
    }

    stream.pause().expect("error pausing stream")
}

fn get_audio_callback(shared_buf: Arc<Mutex<AudioDataBuffer>>) -> impl FnMut(&[f32], &cpal::InputCallbackInfo)
{
    return move |data: &[f32], _: &cpal::InputCallbackInfo| {
        // println!("before");
        let buf = shared_buf.lock();
        let mut mutbuf = buf.unwrap();
        // println!("callback thread: {}", thread::current().name().unwrap_or("shit"));
        // let mut buf = shared_buf.lock().unwrap();
        mutbuf.push_data(data);
        // println!("after {}", mutbuf.len)
    };
}



fn get_audio(data: &[f32], _: &cpal::InputCallbackInfo) {
    let max = IntoIterator::into_iter(data).max_by(|a, b| if a > b { Ordering::Greater } else { Ordering::Less }).unwrap().to_f32();
    let min = IntoIterator::into_iter(data).min_by(|a, b| if a > b { Ordering::Greater } else { Ordering::Less }).unwrap().to_f32();

    println!("Got {} data points; max = {}, min = {}", data.len(), max, min)
}