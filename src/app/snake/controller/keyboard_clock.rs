use crate::{
    app::{
        apple::Apple,
        snake::{
            controller::{Controller, OtherSnakes},
            Body,
        },
    },
    basic::{Dir, Dir12},
};
use ggez::event::KeyCode;
use ggez::Context;
use crate::app::game_context::GameContext;

// joke controller with 12 directions allowing the player to rotate between them using left and right, surprising horizontal teleportation
// looks pretty cool with the sharp drawing style
pub struct KeyboardClock {
    pub dir: Dir12,
    pub alternation: bool,
    pub next_dir: Option<Dir12>,
}

impl Controller for KeyboardClock {
    fn next_dir(&mut self, _: &mut Body, _: OtherSnakes, _: &[Apple], _: &GameContext, _: &Context) -> Option<Dir> {
        if let Some(new_dir) = self.next_dir.take() {
            self.dir = new_dir;
            self.alternation = false;
        }
        match &mut self.dir {
            Dir12::Single(dir) => Some(*dir),
            Dir12::Combined(a, b) => {
                self.alternation = !self.alternation;
                match self.alternation {
                    true => Some(*a),
                    false => Some(*b),
                }
            }
        }
    }

    fn reset(&mut self, dir: Dir) {
        self.next_dir = None;
        self.dir = Dir12::Single(dir);
    }

    fn key_pressed(&mut self, key: KeyCode) {
        match key {
            KeyCode::Left => self.dir -= 1,
            KeyCode::Right => self.dir += 1,
            _ => (),
        }
    }
}
