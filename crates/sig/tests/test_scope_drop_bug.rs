use sig::{create_scope, Signal};

#[test]
fn test_scope_drop_with_signal_access() {
    create_scope(|| {
        // 创建一个 signal 来存储 (node, scope_id)
        let storage: Signal<Vec<(Signal<i32>, sig::reactive::ScopeId)>> = Signal::new(Vec::new());
        
        // 第一次：创建一个子 scope 和 signal
        let (signal1, scope1) = sig::reactive::as_child_scope(|_: ()| {
            Signal::new(42)
        })(());
        
        eprintln!("✅ Created signal in scope {:?}", scope1);
        
        // 存储到 storage
        *storage.write() = vec![(signal1, scope1)];
        
        // 读取 signal1 - 应该成功
        eprintln!("📖 Reading signal1: {}", *signal1.read());
        
        // 第二次：创建新的子 scope
        let (signal2, scope2) = sig::reactive::as_child_scope(|_: ()| {
            Signal::new(100)
        })(());
        
        eprintln!("✅ Created signal in scope {:?}", scope2);
        
        // 替换 storage - 旧的 (signal1, scope1) 被 drop
        let old = std::mem::replace(&mut *storage.write(), vec![(signal2, scope2)]);
        eprintln!("📦 Old storage dropped, contained {} items", old.len());
        drop(old); // 显式 drop
        
        // 删除 scope1
        eprintln!("🗑️  Removing scope1 {:?}", scope1);
        sig::runtime::with_reactive_mut(|state| {
            state.remove_scope(scope1);
        });
        eprintln!("✅ Scope1 removed");
        
        // 现在尝试读取 signal2 - 应该成功
        eprintln!("📖 Reading signal2: {}", *signal2.read());
        
        // 但如果我们尝试读取 signal1 - 应该 panic！
        // eprintln!("📖 Reading signal1 again: {}", *signal1.read());
    });
}

#[test]  
#[should_panic]
fn test_scope_drop_causes_signal_panic() {
    create_scope(|| {
        // 创建一个子 scope 和 signal
        let (signal1, scope1) = sig::reactive::as_child_scope(|_: ()| {
            Signal::new(42)
        })(());
        
        eprintln!("✅ Created signal in scope {:?}", scope1);
        eprintln!("📖 Reading signal before drop: {}", *signal1.read());
        
        // 删除 scope  
        eprintln!("🗑️  Removing scope {:?}", scope1);
        sig::runtime::with_reactive_mut(|state| {
            state.remove_scope(scope1);
        });
        
        // 尝试读取 signal - 应该 panic
        eprintln!("📖 Reading signal after scope drop: {}", *signal1.read());
    });
}
