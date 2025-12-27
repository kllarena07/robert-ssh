use std::{
    collections::HashMap,
    io,
    sync::mpsc::{Receiver, Sender},
    thread,
    time::Duration,
};

use crossterm::event::{KeyCode, KeyEventKind, KeyModifiers};
use image::{ImageReader, Rgb};
use ordered_float::OrderedFloat;
use rand::{Rng, rngs::ThreadRng};
use ratatui::{
    DefaultTerminal, Frame,
    style::Color,
    symbols::Marker,
    widgets::canvas::{Canvas, Points},
};

type PixelMap = HashMap<(OrderedFloat<f64>, OrderedFloat<f64>), Rgb<u8>>;

pub fn load_to_pixel_map(file_name: &str) -> PixelMap {
    let open_expect = format!("Couldn't find {file_name}.");
    let decode_expect = format!("Couldn't decode {file_name}.");

    let img = ImageReader::open(file_name)
        .expect(&open_expect)
        .decode()
        .expect(&decode_expect);
    let img_as_rgb = img.to_rgb8();

    let pixel_map: PixelMap = img_as_rgb
        .enumerate_pixels()
        .map(|(x, y, rgb_val)| {
            let x = f64::from(x);
            let y = f64::from(y);
            let offset = f64::from(y > 1.0) * 0.5;
            let actual_y = y * offset;
            (
                (OrderedFloat(x), OrderedFloat(actual_y)),
                rgb_val.to_owned(),
            )
        })
        .collect::<Vec<((OrderedFloat<f64>, OrderedFloat<f64>), Rgb<u8>)>>() // convert to Vec<((f64, f64), Rgb<u8>)>
        .into_iter()
        .collect::<PixelMap>(); // convert to PixelMap

    pixel_map
}

pub enum Event {
    Input(crossterm::event::KeyEvent),
    Tick(()),
}

pub struct App {
    pub exit: bool,
    pub offset: (f64, f64),
    pub sx: f64,
    pub sy: f64,
    pub normal_pixel_map: PixelMap,
    pub scared_pixel_map: PixelMap,
    pub rng: ThreadRng,
}

pub fn tick(tx: Sender<Event>) {
    loop {
        tx.send(Event::Tick(())).unwrap();
        thread::sleep(Duration::from_millis(1000 / 30));
    }
}

pub fn handle_input_events(tx: Sender<Event>) {
    loop {
        match crossterm::event::read().unwrap() {
            crossterm::event::Event::Key(key_event) => tx.send(Event::Input(key_event)).unwrap(),
            _ => {}
        }
    }
}

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal, rx: Receiver<Event>) -> io::Result<()> {
        while !self.exit {
            match rx.recv().unwrap() {
                Event::Input(key_event) => self.handle_key_event(key_event)?,
                Event::Tick(_) => {}
            }
            terminal.draw(|frame| self.draw(frame))?;
        }
        Ok(())
    }
    pub fn draw(&mut self, frame: &mut Frame) {
        let fa = frame.area();
        let width = f64::from(fa.width);
        let height = f64::from(fa.height);

        self.check_bounds(width, height);
        self.offset.0 += self.sx;
        self.offset.1 += self.sy;

        let canvas = Canvas::default()
            .marker(Marker::HalfBlock)
            .x_bounds([0.0, width])
            .y_bounds([0.0, height])
            .paint(|ctx| {
                let current_map = if self.is_scared() {
                    &self.scared_pixel_map
                } else {
                    &self.normal_pixel_map
                };
                for (coord, rv) in current_map {
                    let x = coord.0;
                    let y = coord.1;
                    let px_offset = self.offset.0;
                    let py_offset = self.offset.1;

                    ctx.draw(&Points {
                        coords: &[(*x - px_offset, height - *y + py_offset)],
                        color: Color::Rgb(rv[0], rv[1], rv[2]),
                    });
                }
            });
        frame.render_widget(canvas, frame.area());
    }
    fn handle_key_event(&mut self, key_event: crossterm::event::KeyEvent) -> io::Result<()> {
        if key_event.kind == KeyEventKind::Press {
            match key_event.code {
                KeyCode::Char('q') => {
                    self.exit = true;
                }
                KeyCode::Char('c') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.exit = true;
                }
                KeyCode::Esc => {
                    self.exit = true;
                }
                _ => {}
            };
        }

        Ok(())
    }
    fn check_bounds(&mut self, width: f64, height: f64) {
        if self.offset.1 > 0.0 {
            self.reverse_sy();
        }
        if self.offset.1 < -(height - 16.0) {
            self.reverse_sy();
        }
        if self.offset.0 < -(width - 32.0) {
            self.reverse_sx();
        }
        if self.offset.0 > 0.0 {
            self.reverse_sx();
        }
    }
    fn generate_magnitude(&mut self, default: f64, is_x: bool) -> f64 {
        let odds = if is_x { 1.0 / 2.0 } else { 1.0 / 5.0 };
        let crazy_value = if is_x { 20.0 } else { 5.0 };
        if self.rng.gen_range(0.0..1.0) < odds {
            crazy_value
        } else {
            default
        }
    }
    fn reverse_sy(&mut self) {
        let magnitude = self.generate_magnitude(1.0, false);
        self.sy = -self.sy.signum() * magnitude;
    }
    fn reverse_sx(&mut self) {
        let magnitude = self.generate_magnitude(1.5, true);
        self.sx = -self.sx.signum() * magnitude;
    }

    fn is_scared(&self) -> bool {
        self.sx.abs() > 2.0 || self.sy.abs() > 2.0
    }
}
