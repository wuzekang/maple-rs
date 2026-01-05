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
  #[cfg(debug_assertions)]
  pub created_at: &'static std::panic::Location<'static>,
}

impl<T> SignalData<T> {
  #[inline]
  pub(crate) fn new_with_caller(value: T, #[allow(unused_variables)] caller: &'static std::panic::Location<'static>) -> Self {
    Self {
      id: SignalId::new(),
      value,
      #[cfg(debug_assertions)]
      created_at: caller,
    }
  }
}

// ============================================================================
// Signal - Reactive signal
// ============================================================================

pub struct Signal<T: 'static> {
  pub(crate) inner: GenerationalBox<SignalData<T>, UnsyncStorage>,
}

impl<T: 'static> std::fmt::Debug for Signal<T> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let mut debug = f.debug_struct("Signal");
    
    if let Ok(guard) = self.inner.try_read() {
      debug.field("id", &guard.id);
      
      #[cfg(debug_assertions)]
      {
        debug.field("created_at", &guard.created_at);
      }
    } else {
      debug.field("state", &"<dropped>");
    }
    
    debug.finish()
  }
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

  /// Try to read signal value (returns None if dropped)
  /// This subscribes to the signal like read()
  #[inline]
  pub fn try_read(&self) -> Option<ReadGuard<T>> {
    match self.inner.try_read() {
      Ok(guard) => {
        RUNTIME.with(|state| {
          if let Some(effect_rc) = state.effects.borrow().last() {
            state.subscribe(guard.id, effect_rc.clone());
          }
        });
        Some(ReadGuard { inner: guard })
      }
      Err(_) => None,
    }
  }

  /// Try to read signal value without subscription (returns None if dropped)
  #[inline]
  pub fn try_read_untracked(&self) -> Option<ReadGuard<T>> {
    self.inner.try_read().ok().map(|inner| ReadGuard { inner })
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
  
  /// Get the location where this signal was created (debug builds only)
  #[cfg(debug_assertions)]
  #[inline]
  pub fn created_at(&self) -> &'static std::panic::Location<'static> {
    self.inner.read().created_at
  }

  /// Create a new signal in current scope
  #[track_caller]
  pub fn new(initial_value: T) -> Self {
    Self::new_with_caller(initial_value, std::panic::Location::caller())
  }
  
  /// Create a new signal with explicit caller location
  pub fn new_with_caller(initial_value: T, caller: &'static std::panic::Location<'static>) -> Self {
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
        .insert(SignalData::new_with_caller(initial_value, caller))
    });

    Self { inner }
  }
}
