extern crate core;

use std::cmp::{max, min};
use std::sync::{Arc, Mutex};
use raylib::color::Color;
use raylib::drawing::RaylibDraw;
use raylib::math::Vector2;
use raylib::prelude::RaylibDrawHandle;
use crate::{AUDIO_BUFFER_LEN, AudioDataBuffer};

const MAX_AUDIO_POINTS_FOR_WAVE: usize = AUDIO_BUFFER_LEN / 2;

pub struct WaveViz {
    width: i32,
    height: i32,

    points: Vec<Vector2>
}

impl WaveViz {
    pub fn new(width: i32, height: i32) -> Self {
        let mut points = vec![Vector2::zero(); min(AUDIO_BUFFER_LEN, (width / 4) as usize)];
        for i in 0..points.len() {
            points[i].x = (width as f32 * (i as f32 / points.len() as f32));
        }

        return Self {
            width,
            height,
            points,
        }
    }

    pub fn render(&mut self, d: &mut RaylibDrawHandle, audio_data: [f32; AUDIO_BUFFER_LEN]) {
        let points_per_vert = min(AUDIO_BUFFER_LEN, MAX_AUDIO_POINTS_FOR_WAVE) / self.points.len();
        let audio_data_start = AUDIO_BUFFER_LEN - min(AUDIO_BUFFER_LEN, MAX_AUDIO_POINTS_FOR_WAVE);

        for i in 0..self.points.len() {
            let mut y = 0.0;
            for p in 0..points_per_vert {
                y += audio_data[audio_data_start + (i*points_per_vert) + p];
            }

            y /= points_per_vert as f32;
            //let y = audio_data[i];
            //let scaled = (y-1.0).powf(3.0) + 1.0;
            // let scaled = -(y-1.0).powf(10.0) + 1.0;
            let scaled = -(y-1.0).powf(2.0) + 1.0;
            self.points[i].y = (scaled * 450.0) + 450.0;

            // if i > 0 {
            //     d.draw_line_ex(self.points[i-1], self.points[i], 3 as f32, Color::PURPLE);
            // }
        }

        for i in 0..self.points.len() {
            if i > 1 && i < self.points.len() - 1 {
                self.points[i].y = (self.points[i-1].y + self.points[i].y*2.0 + self.points[i+1].y) / 4.0
            }

            if i > 0 {
                d.draw_line_ex(self.points[i-1], self.points[i], 3 as f32, Color::PURPLE);
            }
        }

        // d.draw_line_strip(&self.points[..], Color::BLUE)


        //
        // for x in 0..min(AUDIO_BUFFER_LEN, self.width as usize) {
        //     let pt = audio_data[x];
        //
        //     d.draw_circle(x as i32, (pt * 400.0) as i32 + 400, 1.0, Color::GREEN);
        // }
    }
}