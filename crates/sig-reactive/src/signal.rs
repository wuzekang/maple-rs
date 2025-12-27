use generational_box::{AnyStorage, GenerationalBox, UnsyncStorage};
use std::ops::{Deref, DerefMut};
use std::sync::atomic::{AtomicUsize, Ordering};

use super::runtime::RUNTIME;

// ============================================================================
// SignalId - Unique identifier for Signal
// ============================================================================

static SIGNAL_ID_COUNTER: AtomicUsize = AtomicUsize::new(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SignalId(pub usize);

impl SignalId {
  #[inline]
  pub fn new() -> Self {
    Self(SIGNAL_ID_COUNTER.fetch_add(1, Ordering::Relaxed))
  }
}

// ============================================================================
// SignalData - Signal data wrapper
// ============================================================================

pub struct SignalData<T> {
  pub id: SignalId,
  pub value: T,
}

impl<T> SignalData<T> {
  #[inline]
  pub(crate) fn new(value: T) -> Self {
    Self {
      id: SignalId::new(),
      value,
    }
  }
}

// ============================================================================
// Signal - Reactive signal
// ============================================================================

#[derive(Debug)]
pub struct Signal<T: 'static> {
  pub(crate) inner: GenerationalBox<SignalData<T>, UnsyncStorage>,
}

impl<T: 'static> Copy for Signal<T> {}

impl<T: 'static> Clone for Signal<T> {
  fn clone(&self) -> Self {
    *self
  }
}

// ============================================================================
// ReadGuard - Immutable access guard
// ============================================================================

pub struct ReadGuard<T: 'static> {
  pub(crate) inner: <UnsyncStorage as AnyStorage>::Ref<'static, SignalData<T>>,
}

impl<T> Deref for ReadGuard<T> {
  type Target = T;

  #[inline]
  fn deref(&self) -> &Self::Target {
    &self.inner.value
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

// ============================================================================
// WriteGuard - Mutable access guard with auto-notification
// ============================================================================

pub struct WriteGuard<T: 'static> {
  // Write guard holds the mutable reference to the signal data
  // This will be dropped FIRST (because it's declared first)
  pub(crate) inner: <UnsyncStorage as AnyStorage>::Mut<'static, SignalData<T>>,
  // Metadata holds the signal ID and is dropped SECOND
  // When it drops, the signal borrow has already been released
  #[allow(dead_code)]
  pub(crate) metadata: WriteGuardMetadata,
}

/// Metadata for WriteGuard that handles cleanup after the write is complete
pub struct WriteGuardMetadata {
  id: SignalId,
}

impl Drop for WriteGuardMetadata {
  fn drop(&mut self) {
    // At this point, WriteGuard.inner has already been dropped,
    // so the signal borrow is released and we can safely run effects
    RUNTIME.with(|state| {
      let effects = state.get_signal_subscribers(self.id);
      let depth = *state.effect_depth.borrow();
      
      if depth > 0 {
        // We're inside an effect - queue the effects for later (with deduplication)
        for effect in effects {
          state.queue_effect(effect);
        }
      } else {
        // We're at top level - run effects immediately and flush pending queue
        for effect in effects {
          super::Effect::run_with_rc(&effect);
        }
        state.flush_pending_effects();
      }
    });
  }
}

impl<T> Deref for WriteGuard<T> {
  type Target = T;

  #[inline]
  fn deref(&self) -> &Self::Target {
    &self.inner.value
  }
}

impl<T> DerefMut for WriteGuard<T> {
  #[inline]
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.inner.value
  }
}

// ============================================================================
// Signal implementation
// ============================================================================

impl<T: 'static> Signal<T> {
  /// Read signal value (subscribes to current effect)
  #[inline]
  pub fn read(&self) -> ReadGuard<T> {
    let guard = self.inner.read();
    RUNTIME.with(|state| {
      if let Some(effect_rc) = state.effects.borrow().last() {
        state.subscribe(guard.id, effect_rc.clone());
      }
    });
    ReadGuard { inner: guard }
  }

  /// Read signal value without subscription
  #[inline]
  pub fn read_untracked(&self) -> ReadGuard<T> {
    ReadGuard {
      inner: self.inner.read(),
    }
  }

  pub fn write(&self) -> WriteGuard<T> {
    let guard = self.inner.write();
    let id = guard.id;
    
    WriteGuard {
      inner: guard,
      metadata: WriteGuardMetadata { id },
    }
  }

  /// Try to write to signal, returns None if the signal has been dropped
  pub fn try_write(&self) -> Option<WriteGuard<T>> {
    match self.inner.try_write() {
      Ok(guard) => {
        let id = guard.id;
        Some(WriteGuard {
          inner: guard,
          metadata: WriteGuardMetadata { id },
        })
      }
      Err(_) => None,
    }
  }

  /// Set value only if changed
  pub fn set_if_changed(&self, new_value: T)
  where
    T: PartialEq,
  {
    if self.inner.read().value != new_value {
      *self.write() = new_value;
    }
  }

  /// Set value without notification
  #[inline]
  pub fn set_untracked(&self, value: T) {
    self.inner.write().value = value;
  }

  /// Get the signal ID
  #[inline]
  pub fn signal_id(&self) -> SignalId {
    self.inner.read().id
  }

  /// Create a new signal in current scope
  pub fn new(initial_value: T) -> Self {
    let inner = RUNTIME.with(|state| {
      let scope_id = state
        .current_scope_id()
        .expect("No current scope. Use create_scope first.");
      
      state
        .scopes
        .borrow()
        .get(&scope_id)
        .expect("Scope not found")
        .as_ref()
        .expect("Scope is None")
        .owner
        .insert(SignalData::new(initial_value))
    });

    Self { inner }
  }
}
