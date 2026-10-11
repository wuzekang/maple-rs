//! Signal primitive — a thin facade over `samara-signals`.
//!
//! The API mirrors the previous in-house implementation: guard-based
//! `read`/`write` with dependency tracking on tracked reads and subscriber
//! notification on write-guard drop.

use samara_signals as samara;
use slotmap::Key as _;
use std::sync::atomic::{AtomicU64, Ordering};

static SIGNAL_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Unique identifier for a signal (debug/tooling aid).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SignalId(pub u64);

impl SignalId {
  #[inline]
  pub fn new() -> Self {
    Self(SIGNAL_ID_COUNTER.fetch_add(1, Ordering::Relaxed))
  }
}

impl Default for SignalId {
  #[inline]
  fn default() -> Self {
    Self::new()
  }
}

/// A reactive value. `Copy` handle; the value lives in the reactive graph
/// and is owned by the scope current at creation time.
pub struct Signal<T: 'static> {
  inner: samara::Signal<T>,
}

impl<T: 'static> std::fmt::Debug for Signal<T> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let mut s = f.debug_struct("Signal");
    s.field("id", &self.inner.key());
    #[cfg(debug_assertions)]
    s.field("created_at", &self.inner.created_at());
    s.finish()
  }
}

impl<T: 'static> Copy for Signal<T> {}

impl<T: 'static> Clone for Signal<T> {
  fn clone(&self) -> Self {
    *self
  }
}

/// Immutable access guard; subscribes the current effect on creation.
pub struct ReadGuard<T: 'static> {
  inner: samara::SignalReadGuard<'static, T>,
}

impl<T> std::ops::Deref for ReadGuard<T> {
  type Target = T;

  #[inline]
  fn deref(&self) -> &Self::Target {
    &self.inner
  }
}

impl<T, U: ?Sized> AsRef<U> for ReadGuard<T>
where
  T: AsRef<U>,
{
  #[inline]
  fn as_ref(&self) -> &U {
    (&**self).as_ref()
  }
}

/// Mutable access guard; notifies subscribers when dropped.
pub struct WriteGuard<T: 'static> {
  inner: samara::SignalWriteGuard<'static, T>,
}

impl<T> std::ops::Deref for WriteGuard<T> {
  type Target = T;

  #[inline]
  fn deref(&self) -> &Self::Target {
    &self.inner
  }
}

impl<T> std::ops::DerefMut for WriteGuard<T> {
  #[inline]
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.inner
  }
}

impl<T: 'static> Signal<T> {
  /// Create a new signal in the current scope.
  #[track_caller]
  pub fn new(initial_value: T) -> Self {
    Self {
      inner: samara::signal(initial_value),
    }
  }

  /// Create a new signal with an explicit caller location (debug builds;
  /// samara collapses locations to `()` in release).
  #[cfg(debug_assertions)]
  pub fn new_with_caller(
    initial_value: T,
    caller: &'static std::panic::Location<'static>,
  ) -> Self {
    Self {
      inner: samara::Signal::new(initial_value, caller),
    }
  }

  /// Read signal value (subscribes the current effect).
  #[inline]
  pub fn read(&self) -> ReadGuard<T> {
    ReadGuard {
      inner: samara::SignalReadGuard::new(self.inner.key()),
    }
  }

  /// Read signal value without subscription.
  #[inline]
  pub fn read_untracked(&self) -> ReadGuard<T> {
    ReadGuard {
      inner: samara::SignalReadGuard::new_untracked(self.inner.key()),
    }
  }

  /// Read signal value (subscribes the current effect); `None` if the
  /// signal's scope has been disposed.
  #[inline]
  pub fn try_read(&self) -> Option<ReadGuard<T>> {
    if !self.inner.exists() {
      return None;
    }
    Some(self.read())
  }

  /// Write access; `None` if the signal's scope has been disposed.
  /// Subscribers are notified when the guard drops.
  #[inline]
  pub fn try_write(&self) -> Option<WriteGuard<T>> {
    if !self.inner.exists() {
      return None;
    }
    Some(self.write())
  }

  /// Write access; subscribers are notified when the guard drops.
  #[inline]
  pub fn write(&self) -> WriteGuard<T> {
    WriteGuard {
      inner: samara::SignalWriteGuard::new(self.inner.key()),
    }
  }

  /// Overwrite the value (tracked write: notifies subscribers).
  #[inline]
  pub fn set(&self, value: T) {
    self.inner.set(value);
  }

  /// Read a clone of the current value without subscribing.
  #[inline]
  pub fn get_untracked(&self) -> T
  where
    T: Clone,
  {
    self.inner.get_untracked()
  }

  /// Read a clone of the current value, subscribing the current effect.
  #[inline]
  pub fn get(&self) -> T
  where
    T: Clone,
  {
    self.inner.get()
  }

  /// Get the signal's graph identifier.
  #[inline]
  pub fn signal_id(&self) -> SignalId {
    SignalId(self.inner.key().data().as_ffi())
  }

  /// Set value only if changed (skips notification when equal).
  pub fn set_if_changed(&self, new_value: T)
  where
    T: PartialEq,
  {
    if *self.read_untracked() != new_value {
      *self.write() = new_value;
    }
  }

  /// Set value without notifying subscribers.
  #[inline]
  pub fn set_untracked(&self, value: T) {
    self.inner.set_silent(value);
  }

  /// Location where this signal was created (debug builds only, matching
  /// the historical API; samara tracks locations only under
  /// debug_assertions).
  #[cfg(debug_assertions)]
  #[inline]
  pub fn created_at(&self) -> &'static std::panic::Location<'static> {
    self.inner.created_at()
  }
}
