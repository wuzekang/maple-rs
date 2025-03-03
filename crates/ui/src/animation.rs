use std::sync::atomic::{AtomicU64, Ordering};
use reactive::on_cleanup;
use crate::runtime::RUNTIME;

pub fn use_raf<F>(f: F)
where
    F: Fn(f32) + 'static,
{
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let id = COUNTER.fetch_add(1, Ordering::Relaxed);
    RUNTIME
        .with_borrow_mut(move |r| r.animation_frame_callbacks.borrow_mut().insert(id, Box::new(f)));
    on_cleanup(move || {
        RUNTIME.with_borrow_mut(|r| {
            r.animation_frame_callbacks.borrow_mut().remove(&id);
        })
    });
}