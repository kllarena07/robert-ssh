use crossterm::event::{KeyCode, KeyEventKind, KeyModifiers};
use ratatui::{
    DefaultTerminal, Frame, prelude::Buffer, prelude::Rect, style::Color, widgets::Widget,
};
use std::{io, sync::mpsc, thread, time::Duration};

pub enum Event {
    Input(crossterm::event::KeyEvent), // crossterm key input event
    Progress(f64),                     // progress update from the computation thread
}

pub struct App {
    pub exit: bool,
    pub x: u16,
    pub y: u16,
    pub background_progress: f64,
}

/// Simulate a computational heavy task.
pub fn run_background_thread(tx: mpsc::Sender<Event>) {
    let mut counter = 0_f64;
    loop {
        thread::sleep(Duration::from_millis(1000 / 60)); // ~16.67ms for 60fps
        counter = (counter + 1.0) % 60.0;
        tx.send(Event::Progress(counter)).unwrap();
    }
}

impl App {
    /// Main task to be run continuously
    pub fn run(
        &mut self,
        terminal: &mut DefaultTerminal,
        rx: mpsc::Receiver<Event>,
    ) -> io::Result<()> {
        while !self.exit {
            match rx.recv().unwrap() {
                Event::Input(key_event) => self.handle_key_event(key_event)?,
                Event::Progress(progress) => self.background_progress = progress,
            }
            terminal.draw(|frame| self.draw(frame))?;
        }
        Ok(())
    }

    /// Render `self`, as we implemented the Widget trait for &App
    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    /// Actions that should be taken when a key event comes in.
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
