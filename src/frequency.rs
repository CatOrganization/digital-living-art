use std::sync::Arc;
use raylib::color::Color;
use raylib::drawing::{RaylibDraw, RaylibDrawHandle};
use rustfft::{Fft, FftPlanner, num_complex::Complex};
use crate::AUDIO_BUFFER_LEN;

const NUM_FREQUENCY_BARS: usize = 180;

pub struct FrequencyViz {
    width: i32,
    height: i32,

    fft: Arc<dyn Fft<f32>>,
    fft_data: [Complex<f32>; AUDIO_BUFFER_LEN],
    fft_scratch: Vec<Complex<f32>>,

    fft_magnitudes: [f32; NUM_FREQUENCY_BARS]
}

impl FrequencyViz {
    pub fn new(width: i32, height: i32) -> Self {
        let mut planner: FftPlanner<f32> = FftPlanner::new();
        let fft = planner.plan_fft_forward(AUDIO_BUFFER_LEN);
        let scratch_size = fft.get_inplace_scratch_len();


        return Self {
            width, height, fft,
            fft_data: [Complex{ re: 0.0f32, im: 0.0f32 }; AUDIO_BUFFER_LEN],
            fft_scratch: vec![Complex{ re: 0.0f32, im: 0.0f32 }; scratch_size],
            fft_magnitudes: [0.0; NUM_FREQUENCY_BARS],
        }
    }

    pub fn render(&mut self, d: &mut RaylibDrawHandle, audio_data: [f32; AUDIO_BUFFER_LEN]) {
        let mean = audio_data.iter().sum::<f32>() / audio_data.len() as f32;

        for i in 0..audio_data.len() {
            self.fft_data[i].re = audio_data[i];// - mean;
            self.fft_data[i].im = 0.0;
        }

        self.fft.process_with_scratch(&mut self.fft_data, &mut self.fft_scratch);

        let points_per_bar = AUDIO_BUFFER_LEN / 2 / NUM_FREQUENCY_BARS as usize;

        // TODO try chunks function
        for i in 0..NUM_FREQUENCY_BARS {
            let mut y = 0.0;
            for p in 0..points_per_bar {
                let pt = self.fft_data[(i as usize *points_per_bar) + p as usize];
                y += pt.norm();
            }

            //let normalized = 10.0 * p.norm().log10(); //((p.re * p.re) + (p.im * p.im)).sqrt();
            let normalized = (y / points_per_bar as f32) * 4.0;

            self.fft_magnitudes[i] = normalized;
            //d.draw_circle(((i as f32 / num_bars as f32) * self.width as f32) as i32, self.height/2 - y as i32, 5.0, Color::RED);
        }

        let bar_width = self.width as usize / NUM_FREQUENCY_BARS;


        for i in 0..NUM_FREQUENCY_BARS {
            let idx = if i < NUM_FREQUENCY_BARS / 2 {
                i
            } else {
                NUM_FREQUENCY_BARS - i - 1
            };

            let mag =
                if idx == 0 {
                    (self.fft_magnitudes[0]*2.0 + self.fft_magnitudes[1]) / 3.0
                } else if idx > 0 && idx < NUM_FREQUENCY_BARS-1 {
                    (self.fft_magnitudes[idx-1] + (self.fft_magnitudes[idx] * 2.0) + self.fft_magnitudes[idx+1]) / 4.0
                } else {
                    (self.fft_magnitudes[idx-1] + self.fft_magnitudes[idx]*2.0) / 3.0
                } * 2.0;

            d.draw_rectangle((bar_width * i) as i32, 0, (bar_width - 1) as i32, mag as i32 + 50 , Color::PURPLE);
            d.draw_rectangle((bar_width * i) as i32, self.height - (mag + 50.0) as i32, (bar_width - 1) as i32, self.height , Color::PURPLE);
        }

    }
}