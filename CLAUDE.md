## Browser Automation
- 使用playwright mcp 进行浏览器操作

## Rust Development
- 始终在项目根目录执行构建，不要在子crate目录执行

## WebAssembly
- 当前项目 target wasm32-unknown-emscripten
- 不支持 wasm-bindgen web-sys #[wasm_bindgen] wasm-bindgen-futures
- emscripten 环境中对于异步调用不能使用同步包装，会导致死循环
- client wasm 编译使用 ./build.sh
- println 可以正常打印日志

## Emscripten
- emsdk 在 ../emsdk

## Async
- use_context 不能在异步作用域使用
- @crates/ui 基于 thread_local，不支持 Send 不能在异步作用域使用