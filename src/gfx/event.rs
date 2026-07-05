//! Compat for `ggez::event::EventHandler`. Same method shapes (over our
//! `Context`), with no-op defaults so a screen only implements what it needs.

use crate::gfx::input::keyboard::KeyInput;

#[allow(unused_variables)]
pub trait EventHandler<E> {
    fn update(&mut self) -> Result<(), E>;

    fn draw(&mut self) -> Result<(), E>;

    fn key_down_event(&mut self, input: KeyInput, repeated: bool) -> Result<(), E> {
        Ok(())
    }

    fn key_up_event(&mut self, input: KeyInput) -> Result<(), E> {
        Ok(())
    }

    fn mouse_motion_event(&mut self, x: f32, y: f32, dx: f32, dy: f32) -> Result<(), E> {
        Ok(())
    }

    fn resize_event(&mut self, width: f32, height: f32) -> Result<(), E> {
        Ok(())
    }
}
