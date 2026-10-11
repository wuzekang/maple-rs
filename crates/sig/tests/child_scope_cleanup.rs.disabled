use sig::{as_child_scope, create_scope, Signal};

#[test]
fn test_child_scope_basic() {
    create_scope(|| {
        let counter = Signal::new(0);
        
        // 创建一个 child scope 函数
        let scoped_fn = as_child_scope(move |x: i32| {
            let c = *counter.read();
            x + c
        });
        
        // 调用函数，返回 (结果, scope_id)
        let (result1, scope_id1) = scoped_fn(10);
        assert_eq!(result1, 10);
        
        // 修改信号
        *counter.write() = 5;
        
        // 再次调用
        let (result2, scope_id2) = scoped_fn(10);
        assert_eq!(result2, 15);
        
        // 每次调用都创建新的 scope
        assert_ne!(scope_id1, scope_id2);
        
        // 手动清理 scope
        sig::runtime::with_reactive_mut(|state| {
            state.remove_scope(scope_id1);
            state.remove_scope(scope_id2);
        });
    });
}

#[test]
fn test_child_scope_with_signals() {
    create_scope(|| {
        let scoped_fn = as_child_scope(|x: i32| {
            // 在子 scope 中创建 signal
            let internal = Signal::new(x * 10);
            *internal.read()
        });
        
        let (result1, scope_id1) = scoped_fn(1);
        let (result2, scope_id2) = scoped_fn(2);
        let (result3, scope_id3) = scoped_fn(3);
        
        assert_eq!(result1, 10);
        assert_eq!(result2, 20);
        assert_eq!(result3, 30);
        
        // 3 个独立的 scope
        assert_ne!(scope_id1, scope_id2);
        assert_ne!(scope_id2, scope_id3);
        assert_ne!(scope_id1, scope_id3);
        
        // 清理
        sig::runtime::with_reactive_mut(|state| {
            state.remove_scope(scope_id1);
            state.remove_scope(scope_id2);
            state.remove_scope(scope_id3);
        });
    });
}

#[test]
fn test_child_scope_lifecycle() {
    create_scope(|| {
        let scoped_fn = as_child_scope(|x: i32| x * 2);
        
        // 创建一些 scope
        let (_, id1) = scoped_fn(1);
        let (_, id2) = scoped_fn(2);
        let (_, id3) = scoped_fn(3);
        
        // 验证 scope 存在
        sig::runtime::with_reactive(|state| {
            assert!(state.get_scope(id1).is_some());
            assert!(state.get_scope(id2).is_some());
            assert!(state.get_scope(id3).is_some());
        });
        
        // 清理第一个 scope
        sig::runtime::with_reactive_mut(|state| {
            state.remove_scope(id1);
        });
        
        // 验证清理结果
        sig::runtime::with_reactive(|state| {
            assert!(state.get_scope(id1).is_none());
            assert!(state.get_scope(id2).is_some());
            assert!(state.get_scope(id3).is_some());
        });
        
        // 清理剩余
        sig::runtime::with_reactive_mut(|state| {
            state.remove_scope(id2);
            state.remove_scope(id3);
        });
    });
}
