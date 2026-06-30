use macroquad::math::IVec2;
use macroquad::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

/// Returns the unit grid step for a direction.
pub fn direction_delta(d: Direction) -> IVec2 {
    match d {
        Direction::Up => IVec2::new(0, -1),
        Direction::Down => IVec2::new(0, 1),
        Direction::Left => IVec2::new(-1, 0),
        Direction::Right => IVec2::new(1, 0),
    }
}

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

pub struct Input;

impl Default for Input {
    fn default() -> Self {
        Self::new()
    }
}

impl Input {
    pub fn new() -> Self {
        Self
    }

    pub fn is_down(&self, action: Action) -> bool {
        use KeyCode::*;
        match action {
            Action::Up => is_key_down(Up) || is_key_down(W),
            Action::Down => is_key_down(Down) || is_key_down(S),
            Action::Left => is_key_down(Left) || is_key_down(A),
            Action::Right => is_key_down(Right) || is_key_down(D),
            Action::Confirm => is_key_down(Enter) || is_key_down(Space),
            Action::Cancel => is_key_down(Escape),
            Action::Pause => is_key_down(P),
        }
    }

    pub fn is_pressed(&self, action: Action) -> bool {
        use KeyCode::*;
        match action {
            Action::Up => is_key_pressed(Up) || is_key_pressed(W),
            Action::Down => is_key_pressed(Down) || is_key_pressed(S),
            Action::Left => is_key_pressed(Left) || is_key_pressed(A),
            Action::Right => is_key_pressed(Right) || is_key_pressed(D),
            Action::Confirm => is_key_pressed(Enter) || is_key_pressed(Space),
            Action::Cancel => is_key_pressed(Escape),
            Action::Pause => is_key_pressed(P),
        }
    }

    /// The directional key pressed this frame, if any.
    pub fn direction(&self) -> Option<Direction> {
        use KeyCode::*;
        [
            ([Up, W], Direction::Up),
            ([Down, S], Direction::Down),
            ([Left, A], Direction::Left),
            ([Right, D], Direction::Right),
        ]
        .into_iter()
        .find(|(keys, _)| keys.iter().any(|&k| is_key_pressed(k)))
        .map(|(_, d)| d)
    }
}
