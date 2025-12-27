use sig::{create_scope, Signal, create_effect};
use std::cell::Cell;
use std::rc::Rc;

#[test]
fn test_dynamic_scope_cleanup() {
    create_scope(|| {
        let is_open = Signal::new(false);
        let is_open_clone = is_open.clone();
        
        let render_count = Rc::new(Cell::new(0));
        let render_count_clone = render_count.clone();
        
        // 模拟 Dynamic 的行为：当 is_open 改变时，创建新的子 scope
        create_effect(move || {
            let open = *is_open_clone.read();
            render_count_clone.set(render_count_clone.get() + 1);
            
            eprintln!("📊 Effect run #{}, is_open: {}", render_count_clone.get(), open);
            
            // 模拟在子 scope 中创建 view
            let (child_node, child_scope_id) = sig::reactive::as_child_scope(|_: ()| {
                // 在子 scope 中创建一些 signals（模拟 View 的 children 等）
                let _child_signal = Signal::new(vec![1, 2, 3]);
                eprintln!("  🔧 Created child signals in new scope");
                "child_node"
            })(());
            
            eprintln!("  ✅ Child node created in scope {:?}", child_scope_id);
            
            // 这里应该调度旧 scope 的清理，但不能立即清理
        });
        
        eprintln!("\n🔄 Changing is_open to true...");
        *is_open.write() = true;
        
        eprintln!("\n✅ Test completed");
    });
}

#[test] 
fn test_signal_access_after_scope_disposed() {
    create_scope(|| {
        let scope_id = {
            let (_, scope_id) = sig::reactive::as_child_scope(|_: ()| {
                let signal = Signal::new(42);
                signal
            })(());
            scope_id
        };
        
        // 现在尝试删除 scope
        eprintln!("🗑️  Removing scope {:?}", scope_id);
        sig::runtime::with_reactive_mut(|state| {
            state.remove_scope(scope_id);
        });
        
        eprintln!("✅ Scope removed successfully");
    });
}
