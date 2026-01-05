//! Critical test: Effect subscription to child scope Signal

use sig::prelude::*;
use std::rc::Rc;
use std::cell::RefCell;

#[test]
fn test_effect_loses_subscription_when_signal_destroyed() {
    create_scope(|| {
        println!("=== CRITICAL TEST: Effect subscription lifecycle ===\n");
        
        let signal_holder: Rc<RefCell<Option<Signal<i32>>>> = Rc::new(RefCell::new(None));
        let holder_for_effect = signal_holder.clone();
        let effect_values = Rc::new(RefCell::new(Vec::<i32>::new()));
        let ev_clone = effect_values.clone();
        
        // Create effect in parent scope that reads Signal
        println!("1. Creating effect in parent scope...");
        create_effect(move || {
            if let Some(sig) = *holder_for_effect.borrow() {
                let val = *sig.read();  // This subscribes the effect to the Signal
                ev_clone.borrow_mut().push(val);
                println!("   Effect read value: {}", val);
            } else {
                println!("   Effect running - no signal yet");
            }
        });
        
        println!("   Effect values: {:?}\n", *effect_values.borrow());
        
        // Create Signal in child scope
        println!("2. Creating Signal in child scope...");
        let child_scope_id = {
            let holder_clone = signal_holder.clone();
            let (sig, scope_id) = sig::reactive::as_child_scope(move |_: ()| {
                let child_signal = Signal::new(100);
                *holder_clone.borrow_mut() = Some(child_signal);
                child_signal
            })(());
            scope_id
        };
        println!("   Child scope: {:?}", child_scope_id);
        println!("   Effect values: {:?}\n", *effect_values.borrow());
        
        // Write to Signal - should trigger effect
        println!("3. Writing value 200 to Signal...");
        if let Some(sig) = *signal_holder.borrow() {
            *sig.write() = 200;
        }
        println!("   Effect values: {:?}", *effect_values.borrow());
        
        if effect_values.borrow().contains(&200) {
            println!("   ✓ Effect was triggered by write\n");
        } else {
            println!("   ✗ ERROR: Effect was NOT triggered! (BUG)\n");
        }
        
        // Destroy child scope
        println!("4. Destroying child scope...");
        sig::reactive::remove_scope(child_scope_id);
        println!("   Effect values after scope destroyed: {:?}\n", *effect_values.borrow());
        
        // Try to write again - effect should NOT run (Signal is destroyed)
        println!("5. Attempting to write 300 after scope destroyed...");
        if let Some(sig) = *signal_holder.borrow() {
            match sig.try_write() {
                Some(mut guard) => {
                    *guard = 300;
                    println!("   WARNING: Write succeeded (shouldn't happen)");
                }
                None => {
                    println!("   ✓ try_write returned None (Signal destroyed)");
                }
            }
        }
        println!("   Final effect values: {:?}\n", *effect_values.borrow());
        
        if effect_values.borrow().contains(&300) {
            println!("   ✗ ERROR: Effect ran after Signal destroyed!");
        } else {
            println!("   ✓ Effect did not run (correct)");
        }
        
        println!("\n=== Summary ===");
        println!("Effect value history: {:?}", *effect_values.borrow());
        println!("Expected: [] -> [100] -> [100, 200]");
    });
}

#[test]
fn test_view_effect_tracking_with_dynamic_signal_replacement() {
    create_scope(|| {
        println!("=== TEST: View effect tracking when Dynamic Signal is replaced ===\n");
        
        let toggle = Signal::new(0);
        let flatten_calls = Rc::new(RefCell::new(Vec::<String>::new()));
        let fc_clone = flatten_calls.clone();
        
        // Manually track what the View's effect sees
        let view_children_signal_ids = Rc::new(RefCell::new(Vec::<String>::new()));
        let vcsid_clone = view_children_signal_ids.clone();
        
        println!("1. Creating View with nested Dynamic...");
        let v = view().child(
            sig::dynamic(move || {
                let t = *toggle.read();
                let fc = fc_clone.clone();
                println!("   Outer dynamic effect, toggle = {}", t);
                
                sig::dynamic(move || {
                    fc.borrow_mut().push(format!("inner_dynamic_effect_{}", t));
                    println!("     Inner dynamic effect, toggle = {}", t);
                    view().name(format!("child_{}", t))
                })
            })
        );
        
        println!("   View.children Signal ID: {:?}", v.children.signal_id());
        println!("   Flatten calls: {:?}\n", *flatten_calls.borrow());
        
        println!("2. Updating toggle to 1...");
        *toggle.write() = 1;
        println!("   Flatten calls: {:?}\n", *flatten_calls.borrow());
        
        println!("3. Updating toggle to 2...");
        *toggle.write() = 2;
        println!("   Flatten calls: {:?}\n", *flatten_calls.borrow());
        
        // The key question: When outer dynamic updates and recreates inner dynamic,
        // does View's effect properly unsubscribe from the old inner dynamic's Signal
        // and subscribe to the new one?
        
        println!("\n=== Analysis ===");
        println!("Each toggle update should:");
        println!("  1. Destroy old inner dynamic's scope (and its Signal)");
        println!("  2. Create new inner dynamic's scope (with new Signal)");
        println!("  3. View's effect should detect change and re-flatten");
        println!("\nActual flatten calls: {:?}", *flatten_calls.borrow());
    });
}

#[test]
fn test_signal_id_changes_when_scope_recreated() {
    create_scope(|| {
        println!("=== TEST: Signal IDs change when scope is recreated ===\n");
        
        let trigger = Signal::new(0);
        let signal_ids = Rc::new(RefCell::new(Vec::<usize>::new()));
        
        println!("Creating outer dynamic...");
        let dynamic_node = sig::dynamic(move || {
            let t = *trigger.read();
            let ids_clone = signal_ids.clone();
            
            println!("  Outer effect running, trigger = {}", t);
            
            // Each time this runs, inner dynamic is recreated with a new Signal
            let inner_dyn = sig::dynamic(move || {
                println!("    Inner effect running, trigger = {}", t);
                view().name(format!("v_{}", t))
            });
            
            // Try to get the Signal ID from the Dynamic
            // (This is tricky because Dynamic doesn't expose its signal directly)
            println!("    Inner dynamic created");
            
            inner_dyn
        });
        
        println!("\nInitial state");
        
        println!("\nUpdating trigger to 1...");
        *trigger.write() = 1;
        
        println!("\nUpdating trigger to 2...");
        *trigger.write() = 2;
        
        println!("\n=== Conclusion ===");
        println!("Each update creates a new inner dynamic with a new scope and new Signal.");
        println!("The old Signal is destroyed when its scope is removed.");
    });
}
