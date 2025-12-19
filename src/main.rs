// mod app;
// mod player;

// use std::{io, sync::mpsc, thread};

// use crate::player::Player;

// fn main() -> io::Result<()> {
//     let mut terminal = ratatui::init();

//     let (event_tx, event_rx) = mpsc::channel::<app::Event>();

//     let tx_to_input_events = event_tx.clone();
//     thread::spawn(move || {
//         handle_input_events(tx_to_input_events);
//     });

//     let tx_to_background_progress_events = event_tx.clone();
//     thread::spawn(move || {
//         app::run_background_thread(tx_to_background_progress_events);
//     });

//     let players: Vec<Player> = vec![Player { x: 0, y: 0 }];

//     let mut app = app::App {
//         exit: false,
//         players,
//         background_progress: 0.0,
//     };

//     // App runs on the main thread.
//     let app_result = app.run(&mut terminal, event_rx);

//     // Note: If your threads need clean-up (i.e. the computation thread),
//     // you should communicatie to them that the app wants to shut down.
//     // This is not required here, as our threads don't use resources.
//     ratatui::restore();
//     app_result
// }

use std::{collections::HashMap, io, sync::mpsc, thread};

use crossterm::event::{KeyCode, KeyEventKind, KeyModifiers};
use image::{ImageReader, Rgb};
use ratatui::{
    DefaultTerminal, Frame,
    style::Color,
    symbols::Marker,
    widgets::canvas::{Canvas, Points},
};

fn main() -> io::Result<()> {
    let map = ImageReader::open("./map.png")
        .expect("Error: Couldn't find map.png.")
        .decode()
        .expect("Error: Could not decode map.png.");
    let map_as_rgb = map.to_rgb8();
    let pixel_map: HashMap<(u32, u32), Rgb<u8>> = map_as_rgb
        .enumerate_pixels()
        .map(|(x, y, rgb_val)| ((x, y), rgb_val.to_owned()))
        .collect::<Vec<((u32, u32), Rgb<u8>)>>() // convert to Vec<((u32, u32), Rgb<u8>)>
        .into_iter()
        .collect::<HashMap<(u32, u32), Rgb<u8>>>(); // <HashMap<(u32, u32), Rgb<u8>>>

    let mut terminal = ratatui::init();

    let (event_tx, event_rx) = mpsc::channel::<Event>();
    let tx_to_input_events = event_tx.clone();
    thread::spawn(move || {
        handle_input_events(tx_to_input_events);
    });

    let mut app = App {
        exit: false,
        offset: (0.0, 0.0),
        speed: 5.0,
        pixel_map,
    };

    let app_result = app.run(&mut terminal, event_rx);

    ratatui::restore();
    app_result
}

enum Event {
    Input(crossterm::event::KeyEvent),
}

struct App {
    exit: bool,
    offset: (f64, f64),
    speed: f64,
    pixel_map: HashMap<(u32, u32), Rgb<u8>>,
}

fn handle_input_events(tx: mpsc::Sender<Event>) {
    loop {
        match crossterm::event::read().unwrap() {
            crossterm::event::Event::Key(key_event) => tx.send(Event::Input(key_event)).unwrap(),
            _ => {}
        }
    }
}

impl App {
    pub fn run(
        &mut self,
        terminal: &mut DefaultTerminal,
        rx: mpsc::Receiver<Event>,
    ) -> io::Result<()> {
        while !self.exit {
            match rx.recv().unwrap() {
                Event::Input(key_event) => self.handle_key_event(key_event)?,
            }
            terminal.draw(|frame| self.draw(frame))?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        let fa = frame.area();
        let width = f64::from(fa.width);
        let height = f64::from(fa.height);

        let canvas = Canvas::default()
            .marker(Marker::HalfBlock)
            .x_bounds([0.0, width])
            .y_bounds([0.0, height])
            .paint(|ctx| {
                for (coord, rv) in &self.pixel_map {
                    let x = f64::from(coord.0);
                    let y = f64::from(coord.1);
                    let offset = f64::from(y > 1.0) * 0.5;
                    let actual_y = y * offset;

                    // we need to skip all the stuff that's not in view
                    if x > width || height - actual_y < 0.0 {
                        continue;
                    }

                    let px_offset = self.offset.0;
                    let py_offset = self.offset.1;
                    ctx.draw(&Points {
                        coords: &[(x - px_offset, height - actual_y + py_offset)],
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
                KeyCode::Char('w') | KeyCode::Up => {
                    if self.offset.1 > 0.0 {
                        self.offset.1 -= self.speed;
                    }
                }
                KeyCode::Char('a') | KeyCode::Left => {
                    if self.offset.0 > 0.0 {
                        self.offset.0 -= self.speed;
                    }
                }
                KeyCode::Char('s') | KeyCode::Down => {
                    self.offset.1 += self.speed;
                }
                KeyCode::Char('d') | KeyCode::Right => {
                    self.offset.0 += self.speed;
                }
                _ => {}
            };
        }

        Ok(())
    }
}
