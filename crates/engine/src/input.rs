use macroquad::math::IVec2;
use macroquad::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    /// The reverse of this direction.
    pub fn opposite(self) -> Direction {
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }
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
        Self::check(action, is_key_down)
    }

    pub fn is_pressed(&self, action: Action) -> bool {
        Self::check(action, is_key_pressed)
    }

    fn check(action: Action, key_state: impl Fn(KeyCode) -> bool) -> bool {
        use KeyCode::*;
        match action {
            Action::Up => key_state(Up) || key_state(W),
            Action::Down => key_state(Down) || key_state(S),
            Action::Left => key_state(Left) || key_state(A),
            Action::Right => key_state(Right) || key_state(D),
            Action::Confirm => key_state(Enter) || key_state(Space),
            Action::Cancel => key_state(Escape),
            Action::Pause => key_state(P),
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
