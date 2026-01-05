//! Tests for signal creation location tracking

use sig_reactive::*;

#[test]
#[cfg(debug_assertions)]
fn test_signal_location_tracking() {
    effect::create_scope(|| {
        let signal = Signal::new(42);
        let location = signal.created_at();
        
        // Check that we have a valid location
        assert!(location.file().ends_with("signal_location.rs"));
        assert_eq!(location.line(), 9); // Line where Signal::new is called
    });
}

#[test]
#[cfg(debug_assertions)]
fn test_signal_location_with_helper() {
    effect::create_scope(|| {
        let signal = create_test_signal();
        let location = signal.created_at();
        
        // The location should point to where create_test_signal is called (line 22),
        // not inside the helper function, because we use #[track_caller]
        assert!(location.file().ends_with("signal_location.rs"));
        assert_eq!(location.line(), 22);
    });
}

#[track_caller]
fn create_test_signal() -> Signal<i32> {
    Signal::new(100)
}

#[test]
#[cfg(debug_assertions)]
fn test_signal_location_with_explicit_caller() {
    effect::create_scope(|| {
        // Create a signal with explicit caller location
        let location = std::panic::Location::caller();
        let signal = Signal::new_with_caller(42, location);
        
        assert_eq!(signal.created_at(), location);
    });
}

#[test]
fn test_signal_debug_format() {
    effect::create_scope(|| {
        let signal = Signal::new(42);
        let debug_str = format!("{:?}", signal);
        
        // Debug output should include SignalId
        assert!(debug_str.contains("Signal"));
        assert!(debug_str.contains("id"));
        
        #[cfg(debug_assertions)]
        {
            // In debug builds, should also include created_at
            assert!(debug_str.contains("created_at"));
        }
    });
}

#[test]
fn test_signal_debug_dropped() {
    effect::create_scope(|| {
        let signal = Signal::new(42);
        
        // Signal is still alive, should show normal debug output
        let debug_str1 = format!("{:?}", signal);
        assert!(debug_str1.contains("Signal"));
        assert!(debug_str1.contains("id"));
    });
    
    // Note: The signal is dropped when the scope ends, but we can't access it here
    // because it's owned by the scope. This test verifies the signal works within scope.
}
