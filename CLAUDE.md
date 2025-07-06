## Server Management
- 用 tmux mcp 维护后台服务

## Browser Automation
- 使用playwright mcp 进行浏览器操作

## Rust Development
- 始终在项目根目录执行构建，不要在子crate目录执行

## WebAssembly
- 当前项目 target wasm32-unknown-emscripten
- 不支持 wasm-bindgen web-sys #[wasm_bindgen] wasm-bindgen-futures
- emscripten 环境中对于异步调用不能使用同步包装，会导致死循环