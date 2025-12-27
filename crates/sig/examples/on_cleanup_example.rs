// Example demonstrating on_cleanup functionality in sig
// This example shows how to use on_cleanup for resource management

use sig::prelude::*;

fn main() {
  println!("=== on_cleanup Example ===\n");

  example_basic_cleanup();
  example_multiple_cleanups();
  example_nested_scopes();
}

fn example_basic_cleanup() {
  println!("1. Basic Cleanup:");

  create_scope(|| {
    println!("   - Scope created");

    on_cleanup(|| {
      println!("   - Cleanup executed!");
    });

    println!("   - Doing some work...");
  });

  println!("   - Scope ended\n");
}

fn example_multiple_cleanups() {
  println!("2. Multiple Cleanups (LIFO order):");

  create_scope(|| {
    println!("   - Registering cleanups...");

    on_cleanup(|| println!("   - Cleanup 1"));
    on_cleanup(|| println!("   - Cleanup 2"));
    on_cleanup(|| println!("   - Cleanup 3"));
  });

  println!("   - All cleanups done\n");
}

fn example_nested_scopes() {
  println!("3. Nested Scopes:");

  create_scope(|| {
    println!("   - Outer scope created");

    on_cleanup(|| {
      println!("   - Outer cleanup");
    });

    create_scope(|| {
      println!("   - Inner scope created");

      on_cleanup(|| {
        println!("   - Inner cleanup");
      });
    });

    println!("   - Inner scope ended");
  });

  println!("   - Outer scope ended\n");
}
