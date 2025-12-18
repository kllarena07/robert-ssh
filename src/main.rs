use crossterm::event::{KeyCode, KeyEventKind, KeyModifiers};
use ratatui::{
    DefaultTerminal, Frame,
    prelude::{Buffer, Rect},
    style::Color,
    widgets::Widget,
};
use std::{io, sync::mpsc, thread, time::Duration};

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();

    let (event_tx, event_rx) = mpsc::channel::<Event>();

    let tx_to_input_events = event_tx.clone();
    thread::spawn(move || {
        handle_input_events(tx_to_input_events);
    });

    let tx_to_background_progress_events = event_tx.clone();
    thread::spawn(move || {
        run_background_thread(tx_to_background_progress_events);
    });

    let mut app = App {
        exit: false,
        x: 0,
        y: 0,
        background_progress: 0.0,
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
    Input(crossterm::event::KeyEvent),
    Progress(f64),
}

pub struct App {
    exit: bool,
    x: u16,
    y: u16,
    background_progress: f64,
}

fn handle_input_events(tx: mpsc::Sender<Event>) {
    loop {
        match crossterm::event::read().unwrap() {
            crossterm::event::Event::Key(key_event) => tx.send(Event::Input(key_event)).unwrap(),
            _ => {}
        }
    }
}

fn run_background_thread(tx: mpsc::Sender<Event>) {
    let mut counter = 0_f64;
    loop {
        thread::sleep(Duration::from_millis(1000 / 60)); // ~16.67ms for 60fps
        counter = (counter + 1.0) % 60.0;
        tx.send(Event::Progress(counter)).unwrap();
    }
}

impl App {
    fn run(&mut self, terminal: &mut DefaultTerminal, rx: mpsc::Receiver<Event>) -> io::Result<()> {
        while !self.exit {
            match rx.recv().unwrap() {
                Event::Input(key_event) => self.handle_key_event(key_event)?,
                Event::Progress(progress) => self.background_progress = progress,
            }
            terminal.draw(|frame| self.draw(frame))?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_key_event(&mut self, key_event: crossterm::event::KeyEvent) -> io::Result<()> {
        if key_event.kind == KeyEventKind::Press {
            match key_event.code {
                KeyCode::Char('c') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.exit = true;
                }
                KeyCode::Esc => {
                    self.exit = true;
                }
                KeyCode::Char('w') | KeyCode::Up => {
                    self.y = self.y.saturating_sub(1);
                }
                KeyCode::Char('a') | KeyCode::Left => {
                    self.x = self.x.saturating_sub(2);
                }
                KeyCode::Char('s') | KeyCode::Down => {
                    self.y = self.y.saturating_add(1);
                }
                KeyCode::Char('d') | KeyCode::Right => {
                    self.x = self.x.saturating_add(2);
                }
                _ => {}
            };
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
                buf[(x.saturating_add(dx as u16), y.saturating_add(dy as u16))].set_bg(Color::Red);
            }
        }
    }
}
