use sig::prelude::*;

#[test]
fn test_nested_signal_read_dependency_tracking() {
    create_scope(|| {
        // 内层 signal
        let inner_signal = Signal::new(42);

        // 外层 signal（包含内层 signal 的引用）
        let outer_signal = Signal::new(Some(inner_signal.clone()));

        let effect_count = std::rc::Rc::new(std::cell::RefCell::new(0));
        let effect_count_clone = effect_count.clone();

        // 创建 effect 读取外层 signal
        sig::create_effect(move || {
            let count = &mut *effect_count_clone.borrow_mut();
            *count += 1;

            let outer_ref = outer_signal.read();
            if let Some(inner) = outer_ref.as_ref() {
                // 在 effect 闭包中读取内层 signal
                let inner_val = inner.read();
                eprintln!("Effect #{}: inner_val = {}", *count, *inner_val);
            } else {
                eprintln!("Effect #{}: outer is None", *count);
            }
        });

        eprintln!("初始状态: effect 执行了 {} 次", effect_count.borrow());
        assert_eq!(*effect_count.borrow(), 1, "Effect should run once initially");

        // 修改内层 signal
        eprintln!("修改 inner_signal...");
        *inner_signal.write() = 100;
        eprintln!("effect 执行了 {} 次", effect_count.borrow());
        assert_eq!(*effect_count.borrow(), 2, "Effect should run after inner signal write");
    });
}
