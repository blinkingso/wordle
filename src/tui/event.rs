//! Handle keyboards and mouse event here.

use crossterm::event::{KeyEvent, MouseEvent};

#[derive(Debug, Clone, Copy)]
pub enum Event {
    Init,
    Quit,
    Error,
    Tick,
    Render,
    Key(KeyEvent),
    Mouse(MouseEvent),
}
