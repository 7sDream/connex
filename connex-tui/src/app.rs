use std::{
    error::Error,
    time::{Duration, Instant},
};

use crossterm::event::{Event, KeyEvent};
use tui::{backend::Backend, Frame, Terminal};

pub trait App {
    type Output;

    fn on_key(&mut self, key: KeyEvent) -> bool;
    fn on_tick(&mut self);
    fn draw<B: Backend>(&self, f: &mut Frame<B>);
    fn output(self) -> Self::Output;

    fn run<B: Backend>(
        mut self, terminal: &mut Terminal<B>, tick_rate: Duration,
    ) -> Result<Self::Output, Box<dyn Error>>
    where
        Self: Sized,
    {
        let mut last_tick = Instant::now();
        loop {
            terminal.draw(|f| self.draw(f))?;

            let timeout = tick_rate.checked_sub(last_tick.elapsed()).unwrap_or(Duration::ZERO);
            if crossterm::event::poll(timeout)? {
                if let Event::Key(key) = crossterm::event::read()? {
                    if !self.on_key(key) {
                        break;
                    }
                }
            }

            if last_tick.elapsed() >= tick_rate {
                self.on_tick();
                last_tick = Instant::now();
            }
        }

        Ok(self.output())
    }
}
