use glam::{vec2, Vec2};
use sdl3_sys::everything::*;
use std::mem::MaybeUninit;

thread_local! {
    pub (crate) static KEYBOARD_STATE: *const bool = unsafe{
        SDL_GetKeyboardState(std::ptr::null_mut() as * mut core::ffi::c_int)
    };
}

pub fn key_pressed(key: SDL_Scancode) -> bool {
    KEYBOARD_STATE.with(|state| unsafe { *state.offset(key.0 as isize) })
}

pub fn mouse_button_pressed(flags: SDL_MouseButtonFlags) -> bool {
    let mut x = MaybeUninit::uninit();
    let mut y = MaybeUninit::uninit();
    unsafe {
        (flags & SDL_GetMouseState(x.as_mut_ptr(), y.as_mut_ptr())) != 0
    }
}

pub fn mouse_position() -> Vec2 {
    let mut x = MaybeUninit::uninit();
    let mut y = MaybeUninit::uninit();
    unsafe {
        SDL_GetMouseState(x.as_mut_ptr(), y.as_mut_ptr());
        vec2(x.assume_init(), y.assume_init())
    }
}