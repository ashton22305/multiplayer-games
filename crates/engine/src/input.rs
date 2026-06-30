//! Device-agnostic input: games query high-level [`Action`]s and directions
//! instead of raw keys, so keyboard and touch map onto the same game logic.

use macroquad::prelude::*;
use std::cell::Cell;

/// A cardinal direction, the common output of directional input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

/// High-level intents a game reacts to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Up,
    Down,
    Left,
    Right,
    Confirm,
    Cancel,
    Pause,
}

impl Action {
    fn keys(self) -> &'static [KeyCode] {
        use KeyCode::*;
        match self {
            Action::Up => &[Up, W],
            Action::Down => &[Down, S],
            Action::Left => &[Left, A],
            Action::Right => &[Right, D],
            Action::Confirm => &[Enter, Space],
            Action::Cancel => &[Escape],
            Action::Pause => &[P],
        }
    }
}

/// Minimum touch travel (screen pixels) before a drag counts as a swipe.
const SWIPE_THRESHOLD: f32 = 24.0;

/// Per-frame input accessor. Reads live device state from macroquad and tracks
/// in-progress touch swipes across frames.
pub struct Input {
    swipe_start: Cell<Option<Vec2>>,
}

impl Default for Input {
    fn default() -> Self {
        Self::new()
    }
}

impl Input {
    pub fn new() -> Self {
        Self {
            swipe_start: Cell::new(None),
        }
    }

    /// True while any key bound to `action` is held.
    pub fn is_down(&self, action: Action) -> bool {
        action.keys().iter().any(|k| is_key_down(*k))
    }

    /// True on the frame any key bound to `action` transitions to pressed,
    /// or (for Confirm) on a tap.
    pub fn is_pressed(&self, action: Action) -> bool {
        if action.keys().iter().any(|k| is_key_pressed(*k)) {
            return true;
        }
        matches!(action, Action::Confirm) && self.tapped()
    }

    /// The directional intent this frame from the keyboard, if any.
    pub fn direction_pressed(&self) -> Option<Direction> {
        if self.is_pressed(Action::Up) {
            Some(Direction::Up)
        } else if self.is_pressed(Action::Down) {
            Some(Direction::Down)
        } else if self.is_pressed(Action::Left) {
            Some(Direction::Left)
        } else if self.is_pressed(Action::Right) {
            Some(Direction::Right)
        } else {
            None
        }
    }

    /// The directional intent this frame from keyboard or a completed swipe.
    /// Call once per frame; it advances the swipe tracker.
    pub fn requested_direction(&self) -> Option<Direction> {
        let swipe = self.poll_swipe();
        self.direction_pressed().or(swipe)
    }

    /// Whether a tap (short press-and-release with no swipe) completed this frame.
    fn tapped(&self) -> bool {
        // A touch that ends without having moved far is handled inside
        // `poll_swipe`, which returns None for it. To keep taps simple we treat
        // a mouse/touch press as confirm.
        is_mouse_button_pressed(MouseButton::Left)
    }

    fn poll_swipe(&self) -> Option<Direction> {
        let mut result = None;
        for t in touches() {
            match t.phase {
                TouchPhase::Started => self.swipe_start.set(Some(t.position)),
                TouchPhase::Ended | TouchPhase::Cancelled => {
                    if let Some(start) = self.swipe_start.take() {
                        let d = t.position - start;
                        if d.length() >= SWIPE_THRESHOLD {
                            result = Some(if d.x.abs() > d.y.abs() {
                                if d.x > 0.0 {
                                    Direction::Right
                                } else {
                                    Direction::Left
                                }
                            } else if d.y > 0.0 {
                                Direction::Down
                            } else {
                                Direction::Up
                            });
                        }
                    }
                }
                _ => {}
            }
        }
        result
    }
}
