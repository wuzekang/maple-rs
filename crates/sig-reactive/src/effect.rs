//! Effect and scope constructors — facade over `samara-signals`.

use samara_signals as samara;

pub use super::runtime::ScopeId;
pub use super::runtime::Effect;

/// Create a new reactive effect.
///
/// The closure runs once immediately; it re-runs whenever any signal it
/// read (tracked) changes. Dependencies are re-collected on every run.
///
/// Bodies are batched: signal writes inside an effect mark downstream
/// effects dirty but defer their runs (deduplicated) until the outermost
/// effect body completes — sig's historical execution semantics, layered
/// on samara's eager propagation.
pub fn create_effect<F: FnMut() + 'static>(mut f: F) -> Effect {
  Effect {
    inner: samara::effect_recursive(move || {
      samara::start_batch();
      f();
      samara::end_batch();
    }),
  }
}

/// Run `f` inside a fresh child scope, then dispose that scope: cleanups
/// registered inside (directly or by child scopes) run when `f` returns.
pub fn create_scope<F: FnOnce() + 'static>(f: F) {
  samara::scope(f).dispose();
}

/// Create a child-scope wrapper bound to the scope current at call time.
///
/// Each invocation of the returned closure runs the function in a NEW child
/// scope of that captured parent and returns the result together with the
/// scope id, which the caller may later pass to [`super::runtime::remove_scope`].
pub fn as_child_scope<F, Args, R>(f: F) -> impl Fn(Args) -> (R, ScopeId)
where
  F: Fn(Args) -> R + 'static,
  Args: 'static,
  R: 'static,
{
  let scoped = samara::scoped(f);
  move |args: Args| -> (R, ScopeId) {
    let (result, scope) = scoped(args);
    (result, ScopeId { inner: scope })
  }
}
