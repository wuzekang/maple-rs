use reactive::on_cleanup;
use crate::runtime::RUNTIME;

pub fn use_raf<F>(f: F)
where
    F: Fn(f32) + 'static,
{
    let key = RUNTIME
        .with_borrow_mut(move |r| r.animation_frame_callbacks.borrow_mut().insert(Box::new(f)));
    on_cleanup(move || {
        RUNTIME.with_borrow_mut(|r| {
            r.animation_frame_callbacks.borrow_mut().remove(key);
        })
    });
}