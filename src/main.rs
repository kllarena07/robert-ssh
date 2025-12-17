use std::{
    collections::HashMap,
    io,
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

use crossterm::event::{KeyCode, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout},
    prelude::{Buffer, Rect},
    style::{Color, Style, Stylize},
    symbols::border,
    text::Line,
    widgets::{Block, Gauge, Widget},
};

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();

    // Create the channel via which the events will be sent to the main app.
    let (event_tx, event_rx) = mpsc::channel::<Event>();

    // Thread to listen for input events.
    let tx_to_input_events = event_tx.clone();
    thread::spawn(move || {
        handle_input_events(tx_to_input_events);
    });

    let mut app = App {
        exit: false,
        x: 0,
        y: 0,
        x_f: 0.0,
        y_f: 0.0,
        vx: 0.0,
        vy: 0.0,
        pressed_keys: HashMap::new(),
    };

    // App runs on the main thread.
    let app_result = app.run(&mut terminal, event_rx);

    // Note: If your threads need clean-up (i.e. the computation thread),
    // you should communicatie to them that the app wants to shut down.
    // This is not required here, as our threads don't use resources.
    ratatui::restore();
    app_result
}

// Events that can be sent to the main thread.
enum Event {
    Input(crossterm::event::KeyEvent), // crossterm key input event
}

pub struct App {
    exit: bool,
    x: u16,
    y: u16,
    x_f: f32,
    y_f: f32,
    vx: f32,
    vy: f32,
    pressed_keys: HashMap<KeyCode, Instant>,
}

/// Block, waiting for input events from the user.
fn handle_input_events(tx: mpsc::Sender<Event>) {
    loop {
        match crossterm::event::read().unwrap() {
            crossterm::event::Event::Key(key_event) => tx.send(Event::Input(key_event)).unwrap(),
            _ => {}
        }
    }
}

impl App {
    /// Main task to be run continuously
    fn run(&mut self, terminal: &mut DefaultTerminal, rx: mpsc::Receiver<Event>) -> io::Result<()> {
        while !self.exit {
            // Handle all pending events
            while let Ok(event) = rx.try_recv() {
                if let Event::Input(key_event) = event {
                    self.handle_key_event(key_event)?;
                }
            }
            // Update and draw
            terminal.draw(|frame| {
                let area = frame.area();
                self.update(area);
                self.draw(frame);
            })?;
            thread::sleep(Duration::from_millis(16));
        }
        Ok(())
    }

    /// Update the app state
    fn update(&mut self, area: Rect) {
        let speed_x = 0.5;
        let speed_y = 0.35;
        let now = Instant::now();
        self.pressed_keys
            .retain(|_, time| now.duration_since(*time) < Duration::from_millis(100));
        self.vx = 0.0;
        self.vy = 0.0;
        for (key, _) in &self.pressed_keys {
            match key {
                KeyCode::Char('w') => self.vy = -speed_y,
                KeyCode::Char('s') => self.vy = speed_y,
                KeyCode::Char('a') => self.vx = -speed_x,
                KeyCode::Char('d') => self.vx = speed_x,
                _ => {}
            }
        }
        self.x_f += self.vx;
        self.y_f += self.vy;
        self.x_f = self.x_f.max(0.0).min(area.width as f32 - 2.0);
        self.y_f = self.y_f.max(0.0).min(area.height as f32 - 1.0);
        self.x = self.x_f as u16;
        self.y = self.y_f as u16;
    }

    /// Render `self`, as we implemented the Widget trait for &App
    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    /// Actions that should be taken when a key event comes in.
    fn handle_key_event(&mut self, key_event: crossterm::event::KeyEvent) -> io::Result<()> {
        match key_event.code {
            KeyCode::Char('q') if key_event.kind == KeyEventKind::Press => self.exit = true,
            KeyCode::Char(c) if "wasd".contains(c) => {
                if key_event.kind == KeyEventKind::Press {
                    self.pressed_keys.insert(key_event.code, Instant::now());
                }
            }
            _ => {}
        }
        Ok(())
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let x = self.x.min(area.width.saturating_sub(2));
        let y = self.y.min(area.height.saturating_sub(1));
        for dx in 0..2 {
            for dy in 0..1 {
                buf[(x + dx, y + dy)].set_bg(Color::Red);
            }
        }
    }
}
