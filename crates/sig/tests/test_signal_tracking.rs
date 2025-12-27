use sig::prelude::*;

#[test]
fn test_signal_dependency_tracking() {
    create_scope(|| {
        let signal = Signal::new(vec![1, 2, 3]);

        let effect_count = std::rc::Rc::new(std::cell::RefCell::new(0));
        let effect_count_clone = effect_count.clone();

        // 创建 effect 读取 signal
        sig::create_effect(move || {
            let count = &mut *effect_count_clone.borrow_mut();
            *count += 1;
            let value = signal.read();
            eprintln!("Effect #{}: signal.len() = {}", *count, value.len());
        });

        eprintln!("初始状态: effect 执行了 {} 次", effect_count.borrow());
        assert_eq!(*effect_count.borrow(), 1, "Effect should run once initially");

        // 修改 signal 的内容
        eprintln!("修改 signal 内容（替换整个 Vec）...");
        *signal.write() = vec![1, 2, 3, 4];
        eprintln!("effect 执行了 {} 次", effect_count.borrow());
        assert_eq!(*effect_count.borrow(), 2, "Effect should run after first write");

        // 再次修改
        eprintln!("再次修改 signal 内容...");
        *signal.write() = vec![5, 6];
        eprintln!("effect 执行了 {} 次", effect_count.borrow());
        assert_eq!(*effect_count.borrow(), 3, "Effect should run after second write");
    });
}
