use crate::player::Player;
use crossterm::event::{KeyCode, KeyEventKind, KeyModifiers};
use ratatui::{DefaultTerminal, Frame, prelude::Buffer, prelude::Rect, widgets::Widget};
use std::{io, sync::mpsc, thread, time::Duration};

pub enum Event {
    Input(crossterm::event::KeyEvent), // crossterm key input event
    Progress(f64),                     // progress update from the computation thread
}

pub struct App {
    pub exit: bool,
    pub players: Vec<Player>,
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
                    for player in &mut self.players {
                        player.y = player.y.saturating_sub(1);
                    }
                }
                KeyCode::Char('a') | KeyCode::Left => {
                    for player in &mut self.players {
                        player.x = player.x.saturating_sub(2);
                    }
                }
                KeyCode::Char('s') | KeyCode::Down => {
                    for player in &mut self.players {
                        player.y = player.y.saturating_add(1);
                    }
                }
                KeyCode::Char('d') | KeyCode::Right => {
                    for player in &mut self.players {
                        player.x = player.x.saturating_add(2);
                    }
                }
                _ => {}
            };
        }

        Ok(())
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        for player in &self.players {
            player.render(area, buf);
        }
    }
}
