use sig_reactive::*;
use std::rc::Rc;
use std::cell::RefCell;

#[test]
fn test_zombie_effect_prevention() {
    let run_count = Rc::new(RefCell::new(0));
    let run_count_clone = run_count.clone();

    // 外部 trigger
    let trigger = Signal::new(0);
    
    // 用于捕获 ScopeId
    let child_scope_id = Rc::new(RefCell::new(None));
    let child_scope_id_clone = child_scope_id.clone();

    // 1. 设置场景
    // 使用 as_child_scope 创建一个由我们控制的 Scope
    let (_, scope_id) = as_child_scope(move |()| {
        // 内部 Signal
        let internal_data = Signal::new("alive".to_string());
        
        // 内部 Effect
        let run_count = run_count_clone.clone();
        create_effect(move || {
            trigger.read(); // 订阅 trigger
            
            // 如果是 zombie effect，这里会读取已销毁的 Signal -> Panic
            // 修复前：Scope 销毁后 Signal 被 drop，但 Effect 仍在队列中被执行
            // 修复后：Scope 销毁后 Effect 也被 drop (Weak)，执行时被跳过
            let _ = internal_data.read(); 
            
            *run_count.borrow_mut() += 1;
        });
    })(());
    
    *child_scope_id.borrow_mut() = Some(scope_id);

    assert_eq!(*run_count.borrow(), 1, "Initial run");

    // 2. 触发 Zombie 场景
    // 我们必须在一个 Effect 中执行 "Set + Destroy"，这样 Set 产生的 Effect 才会进入 Queue 而不是立即执行
    create_effect(move || {
        // 触发更新 -> 内部 Effect 被加入 Queue (因为我们在 Effect 内，depth > 0)
        *trigger.write() = 1;
        
        // 立即销毁 Scope -> 内部 Signal 被 Drop
        if let Some(id) = *child_scope_id_clone.borrow() {
            remove_scope(id);
        }
    }); 
    // Effect 结束 -> Runtime Flush -> 尝试运行内部 Effect
    // 延迟销毁模式下：Scope 依然存活 -> Effect 执行 -> run_count + 1
    // 然后 Scope 才会被销毁
    
    assert_eq!(*run_count.borrow(), 2, "Effect runs once more before deferred destruction");
}
