use web_fetch::{WebFetch, FetchOptions};
use std::collections::HashMap;

// 使用 Emscripten 运行时
#[cfg(target_os = "emscripten")]
use web_fetch::emscripten_runtime::EmscriptenExecutor;

macro_rules! console_log {
    ($($arg:tt)*) => {
        println!($($arg)*);
    };
}

/// 演示新版本的简洁 Web Fetch Future API
async fn demo_future_api() {
    println!("🚀 演示新版本的简洁 Web Fetch Future API");
    println!("目标平台: {}", std::env::consts::OS);
    
    // 测试 1: 基本 GET 请求
    println!("\n1. 测试基本 GET 请求");
    match WebFetch::get("https://httpbin.org/get").await {
        Ok(response) => {
            println!("✅ GET 成功: status={}, size={} bytes", response.status, response.data.len());
            if !response.data.is_empty() {
                let preview = String::from_utf8_lossy(&response.data[..std::cmp::min(100, response.data.len())]);
                println!("响应预览: {}", preview);
            }
        }
        Err(e) => println!("❌ GET 失败: {}", e),
    }
    
    // 测试 2: 基本 POST 请求
    println!("\n2. 测试基本 POST 请求");
    let json_data = r#"{"name": "test", "message": "Hello from new API!"}"#;
    match WebFetch::post("https://httpbin.org/post", json_data).await {
        Ok(response) => {
            println!("✅ POST 成功: status={}, size={} bytes", response.status, response.data.len());
        }
        Err(e) => println!("❌ POST 失败: {}", e),
    }
    
    // 测试 3: 带自定义选项的请求
    println!("\n3. 测试带自定义选项的请求");
    let mut headers = HashMap::new();
    headers.insert("User-Agent".to_string(), "web-fetch-clean/1.0".to_string());
    headers.insert("Accept".to_string(), "application/json".to_string());
    
    let options = FetchOptions {
        method: "GET".to_string(),
        headers,
        body: None,
        timeout_ms: Some(10000), // 10 秒超时
    };
    
    match WebFetch::fetch("https://httpbin.org/headers", Some(options)).await {
        Ok(response) => {
            println!("✅ 自定义选项请求成功: status={}", response.status);
            println!("URL: {}", response.url);
            println!("状态文本: {}", response.status_text);
        }
        Err(e) => println!("❌ 自定义选项请求失败: {}", e),
    }
    
    // 测试 4: PUT 请求
    println!("\n4. 测试 PUT 请求");
    let put_data = r#"{"updated": true, "timestamp": "2024-01-01"}"#;
    let mut put_headers = HashMap::new();
    put_headers.insert("Content-Type".to_string(), "application/json".to_string());
    
    let put_options = FetchOptions {
        method: "PUT".to_string(),
        headers: put_headers,
        body: Some(put_data.to_string()),
        timeout_ms: Some(15000),
    };
    
    match WebFetch::fetch("https://httpbin.org/put", Some(put_options)).await {
        Ok(response) => {
            println!("✅ PUT 成功: status={}, size={} bytes", response.status, response.data.len());
        }
        Err(e) => println!("❌ PUT 失败: {}", e),
    }
    
    // 测试 5: 并发请求
    println!("\n5. 测试并发请求");
    
    // 在 Emscripten 环境中简化并发请求处理
    println!("并发请求在 Emscripten 环境中需要特殊处理");
    
    // 顺序执行请求
    match WebFetch::get("https://httpbin.org/json").await {
        Ok(r) => println!("  请求 (json): ✅ status={}", r.status),
        Err(e) => println!("  请求 (json): ❌ {}", e),
    }
    
    // 测试 6: 错误处理
    println!("\n6. 测试错误处理");
    match WebFetch::get("https://httpbin.org/status/404").await {
        Ok(response) => {
            println!("收到响应: status={} (这是预期的 404 响应)", response.status);
        }
        Err(e) => println!("错误处理测试: {}", e),
    }
    
    println!("\n🎉 Future API 演示完成!");
    println!("特点:");
    println!("  - 简洁的 API: 只有 get(), post(), fetch() 三个主要方法");
    println!("  - 基于 Channel: 无 ID 系统，自动资源管理");
    println!("  - Future 支持: 原生 async/await 支持");
    println!("  - 并发友好: 每个请求独立，支持并发执行");
}

fn main() {
    println!("Web Fetch Future API 演示程序");
    
    println!("在 Emscripten 环境中运行");
    println!("可以通过 JavaScript 调用:");
    println!("  - Module._run_future_demo()");
    println!("  - Module._check_platform_support()");
    println!("  - Module._test_sync_fetch()");
    
    #[cfg(target_os = "emscripten")]
    {
        println!("\n启动异步运行时...");
        let executor = EmscriptenExecutor::new();
        
        // 生成异步任务
        executor.spawn(demo_future_api());
        
        println!("执行器已启动，进入事件循环...");
        executor.run(); // 这会接管控制流
    }
    
    #[cfg(not(target_os = "emscripten"))]
    {
        println!("注意: 完整的异步支持需要在 Emscripten 环境中运行");
    }
}

// 导出函数供浏览器调用  
#[no_mangle]
pub extern "C" fn run_future_demo() {
    console_log!("启动 Future Demo...");
    
    #[cfg(target_os = "emscripten")]
    {
        let executor = EmscriptenExecutor::new();
        
        executor.spawn(async {
            console_log!("🚀 开始异步演示");
            
            // 测试 GET 请求
            console_log!("\n1. 测试 GET 请求");
            match WebFetch::get("https://httpbin.org/get").await {
                Ok(response) => {
                    console_log!("✅ GET 成功: status={}, size={} bytes", 
                        response.status, response.data.len());
                }
                Err(e) => {
                    console_log!("❌ GET 失败: {:?}", e);
                }
            }
            
            // 测试 POST 请求
            console_log!("\n2. 测试 POST 请求");
            match WebFetch::post("https://httpbin.org/post", r#"{"test": "data"}"#).await {
                Ok(response) => {
                    console_log!("✅ POST 成功: status={}", response.status);
                }
                Err(e) => {
                    console_log!("❌ POST 失败: {:?}", e);
                }
            }
            
            console_log!("\n🎉 异步演示完成！");
        });
        
        console_log!("执行器已启动...");
        executor.run(); // 进入事件循环
    }
    
    #[cfg(not(target_os = "emscripten"))]
    {
        console_log!("此功能仅在 Emscripten 环境中可用");
    }
}

// 检查平台支持
#[no_mangle]
pub extern "C" fn check_platform_support() -> bool {
    true // 始终返回 true，因为已为 Emscripten 优化
}

// 同步测试函数（用于验证 WebFetch API）
#[no_mangle]
pub extern "C" fn test_sync_fetch() -> i32 {
    console_log!("测试 WebFetch Future 创建");
    
    // 测试 Future 的创建（但不执行）
    let _future = WebFetch::get("https://httpbin.org/get");
    console_log!("✅ WebFetch Future 创建成功");
    
    // 返回成功状态
    200
}