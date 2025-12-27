//! Basics - Scope, Signal and Fragment
//!
//! Demonstrates core concepts of sig framework:
//! - Scope lifecycle management
//! - Signal reactivity
//! - Fragment composition

use sig::{Signal, create_scope, fragment};

fn main() {
  // ==================== Scope ====================
  println!("【1】Scope lifecycle management");
  println!("  Scope manages lifecycle of signals and effects\n");

  create_scope(|| {
    // Create signal in scope
    let s = Signal::new(5);
    println!("  ✓ Created signal s = {}", *s.read());

    // Create effect that runs when s changes
    let _effect = sig::create_effect(move || {
      println!("  → Effect triggered: s = {}", *s.read());
    });

    println!("  ✓ Update s → 10");
    *s.write() = 10;

    println!("  ✓ Update s → 20");
    *s.write() = 20;
  });

  println!("\n  ✓ Scope ended, all signals and effects cleaned up\n");

  // ==================== Fragment ====================
  println!("【2】Fragment composition");
  println!("  Fragment combines values without extra hierarchy\n");

  create_scope(|| {
    // Empty fragment
    let empty_frag = fragment(());
    println!(
      "  ✓ Empty fragment: {} children",
      match &empty_frag.children {
        sig::Node::Fragment(nodes) => nodes.len(),
        _ => 0,
      }
    );

    // Single view
    let single_frag = fragment(sig::view());
    println!(
      "  ✓ Single element fragment: {}",
      match &single_frag.children {
        sig::Node::View(_) => "single view",
        _ => "other",
      }
    );

    // Multiple elements
    let multi_frag = fragment((sig::view(), sig::view(), sig::view()));
    println!(
      "  ✓ Multiple elements fragment: {} children",
      match &multi_frag.children {
        sig::Node::Fragment(nodes) => nodes.len(),
        _ => 0,
      }
    );
  });

  // ==================== Signal ====================
  println!("\n【3】Signal reactivity");
  println!("  Signal provides reactive state management\n");

  create_scope(|| {
    let counter = Signal::new(0);
    println!("  ✓ Created counter: {}", *counter.read());

    let _effect = sig::create_effect(move || {
      let value = *counter.read();
      println!("  → Counter changed: {}", value);
    });

    println!("  ✓ Counter + 1");
    *counter.write() += 1;

    println!("  ✓ Counter + 5");
    *counter.write() += 5;
  });
}
