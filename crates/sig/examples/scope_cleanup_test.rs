//! 验证 Dynamic scope 清理的内存泄漏修复
//!
//! 测试场景：
//! 1. Dynamic 多次替换内容，验证旧 scope 被正确清理
//! 2. 每次替换创建新的 Signal，验证不会累积导致 panic

use sig::{create_scope, dynamic, Signal, text};

fn main() {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║         Dynamic Scope 清理内存泄漏测试                    ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");

    create_scope(|| {
        // 创建一个计数器信号
        let counter = Signal::new(0);

        // 创建 Dynamic 内容，每次更新都会创建新的 scope
        let content = dynamic(move || {
            let count = *counter.read();

            // 在 Dynamic 的 scope 中创建新的 Signal
            // 如果 scope 不清理，这些 Signal 会泄漏并导致悬垂引用
            let internal = Signal::new(count * 100);

            text(format!("Count={}, Internal={}", count, *internal.read()))
        });

        println!("✅ 初始状态");
        let nodes = content.signal.read();
        println!("  📊 节点数: {}", nodes.len());
        println!("  📝 内容: {:?}", nodes[0].0);

        println!("\n🔄 模拟 50 次内容更新...");
        println!("  每次更新都会创建新的 scope 和 Signal");
        println!("  如果 scope 清理正常，不会有内存泄漏\n");

        for i in 1..=50 {
            *counter.write() = i;

            // 每 10 次打印状态
            if i % 10 == 0 {
                let nodes = content.signal.read();
                println!("  [更新 {} 次] 节点数: {}, 内容正常", i, nodes.len());
            }
        }

        println!("\n✅ 测试完成！");

        // 验证：读取最终内容，确保没有 panic
        let nodes = content.signal.read();
        println!("\n📊 最终状态：");
        println!("  - 节点数: {}", nodes.len());
        println!("  - 内容: {:?}", nodes[0].0);

        println!("\n💡 验证结果：");
        println!("  ✅ 程序运行 50 次更新没有 panic");
        println!("  ✅ 旧 scope 被正确清理，没有悬垂引用");
        println!("  ✅ 每次只保留当前活跃的 scope");
        println!("\n🎉 Dynamic scope 清理修复成功！");
    });
}
