//! IntoElement trait example
//!
//! This example demonstrates how to use the IntoElement trait to convert various types into ValueNode

use sig::prelude::*;

fn main() {
    println!("IntoElement trait example");
    println!("====================\n");

    create_scope(|| {
        // 1. String conversion
        println!("1. String conversion:");
        let text_node = "Hello, World!".into_element();
        println!("   &str -> ValueNode: {:?}\n", matches!(text_node, sig::Node::View(_)));

        // 2. Number conversion
        println!("2. Number conversion:");
        let num_node = 42.into_element();
        println!("   i32 -> ValueNode: {:?}\n", matches!(num_node, sig::Node::View(_)));

        // 3. Float conversion
        println!("3. Float conversion:");
        let float_node = 3.14.into_element();
        println!("   f64 -> ValueNode: {:?}\n", matches!(float_node, sig::Node::View(_)));

        // 4. Boolean conversion
        println!("4. Boolean conversion:");
        let bool_node = true.into_element();
        println!("   bool -> ValueNode: {:?}\n", matches!(bool_node, sig::Node::View(_)));

        // 5. Vec conversion
        println!("5. Vec conversion:");
        let vec_node = vec!["A", "B", "C"].into_element();
        if let sig::Node::Fragment(children) = vec_node {
            println!("   Vec<&str> -> Fragment with {} children\n", children.len());
        }

        // 6. Option conversion
        println!("6. Option conversion:");
        let some_node = Some("has value").into_element();
        let none_node: sig::Node = None::<&str>.into_element();
        println!("   Some -> ValueNode: {:?}", matches!(some_node, sig::Node::View(_)));
        println!("   None -> Empty Fragment: {:?}\n", matches!(none_node, sig::Node::Fragment(_)));

        // 7. Using in real UI
        println!("7. Using IntoElement in real UI:");
        println!("   These types can be directly used in view().child()!");
    });
}
