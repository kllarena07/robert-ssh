use std::{io, sync::mpsc::channel, thread};

use rand::thread_rng;

mod app;
use crate::app::{App, Event, handle_input_events, load_to_pixel_map, tick};

fn main() -> io::Result<()> {
    let normal_pixel_map = load_to_pixel_map("./normal.png");
    let scared_pixel_map = load_to_pixel_map("./scared.png");

    let mut terminal = ratatui::init();

    let (event_tx, event_rx) = channel::<Event>();
    let tx_to_input_events = event_tx.clone();
    thread::spawn(move || {
        handle_input_events(tx_to_input_events);
    });

    let tx_to_tick = event_tx.clone();
    thread::spawn(move || {
        tick(tx_to_tick);
    });

    let rng = thread_rng();
    let mut app = App {
        exit: false,
        offset: (0.0, 0.0),
        sx: -1.5,
        sy: -1.0,
        normal_pixel_map,
        scared_pixel_map,
        rng,
    };

    let app_result = app.run(&mut terminal, event_rx);

    ratatui::restore();
    app_result
}
