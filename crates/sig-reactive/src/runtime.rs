//! Runtime-level handles and context functions — facade over `samara-signals`.
//!
//! samara keeps the reactive graph (nodes, links, scopes, effects) in a
//! thread-local system; these wrappers expose the small surface sig needs:
//! scope identification/disposal, effect context capture for async work,
//! cleanup registration and untracked execution.

use samara_signals as samara;

/// Identifier of a reactive scope (or effect — an effect IS a scope).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ScopeId {
  pub(crate) inner: samara::Scope,
}

impl ScopeId {
  /// The raw graph key behind this scope.
  pub fn key(&self) -> samara::NodeKey {
    self.inner.key()
  }
}

/// Handle to a reactive effect. `Copy`; dispose via [`Effect::dispose`].
#[derive(Debug, Clone, Copy)]
pub struct Effect {
  pub(crate) inner: samara::Effect,
}

impl Effect {
  /// Dispose the effect: runs its cleanups, unlinks its dependencies and
  /// removes nodes it owns. Idempotent.
  pub fn dispose(&self) {
    self.inner.dispose();
  }

  /// Enter this effect's tracking context; the guard restores the previous
  /// context on drop. Signal reads while entered subscribe to this effect —
  /// the integration point for driving reactive code from a Future poll or
  /// other callback.
  pub fn enter_context(&self) -> samara::EffectContextGuard {
    self.inner.enter_context()
  }
}

/// The scope current at this moment, if a reactive context is active.
///
/// samara always has a root scope, so this never returns `None` in
/// practice; `Some` is preserved for API compatibility.
pub fn current_scope_id() -> Option<ScopeId> {
  Some(ScopeId {
    inner: samara::current_scope(),
  })
}

/// The effect currently tracking signal reads, if any.
pub fn current_effect() -> Option<Effect> {
  samara::current_effect().map(|inner| Effect { inner })
}

/// Dispose a scope by id: cleanups run (children first), owned nodes are
/// unlinked and removed. Safe to call twice.
pub fn remove_scope(scope_id: ScopeId) {
  scope_id.inner.dispose();
}

/// Register a cleanup callback for the current scope.
///
/// Panics outside any reactive scope (samara semantics).
pub fn on_cleanup<F: FnOnce() + 'static>(f: F) {
  samara::on_cleanup(f);
}

/// Run `f` without dependency tracking: signal reads inside `f` do not
/// subscribe the current effect. The current scope is preserved.
pub fn untrack<F, R>(f: F) -> R
where
  F: FnOnce() -> R,
{
  samara::untrack(f)
}
