# web-fetch

A Rust library for making HTTP requests in WebAssembly/Emscripten environments with async/await support.

## Features

- 🚀 **Simple API**: Just three main methods - `get()`, `post()`, and `fetch()`
- ⚡ **True Async/Await**: Native Future support with custom Emscripten runtime
- 🔧 **Zero Configuration**: Works out of the box with Emscripten
- 📦 **Lightweight**: Minimal dependencies, efficient memory usage
- 🎯 **Type Safe**: Full Rust type system with compile-time checks

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
web-fetch = "0.1.0"
```

## Quick Start

```rust
use web_fetch::{WebFetch, FetchOptions};

// Simple GET request
let response = WebFetch::get("https://api.example.com/data").await?;
println!("Status: {}", response.status);
println!("Body: {:?}", response.data);

// POST request with data
let response = WebFetch::post(
    "https://api.example.com/users",
    r#"{"name": "John", "age": 30}"#
).await?;

// Custom request with options
let mut headers = HashMap::new();
headers.insert("Authorization".to_string(), "Bearer token123".to_string());

let options = FetchOptions {
    method: "PUT".to_string(),
    headers,
    body: Some(r#"{"status": "active"}"#.to_string()),
    timeout_ms: Some(5000),
};

let response = WebFetch::fetch("https://api.example.com/user/123", Some(options)).await?;
```

## Emscripten Async Runtime

This crate includes a custom async runtime optimized for Emscripten:

```rust
use web_fetch::emscripten_runtime::EmscriptenExecutor;

fn main() {
    let executor = EmscriptenExecutor::new();
    
    executor.spawn(async {
        let response = WebFetch::get("https://httpbin.org/get").await;
        println!("Got response: {:?}", response);
    });
    
    executor.run(); // Starts the event loop
}
```

### Runtime Features

- **Efficient Task Scheduling**: Only polls tasks that are ready
- **Low Latency**: 60 FPS polling rate (16ms max delay)
- **Memory Efficient**: Reuses task slots to minimize allocations
- **Browser Integration**: Works seamlessly with JavaScript event loop

## Building for WebAssembly

1. Install Emscripten SDK:
```bash
git clone https://github.com/emscripten-core/emsdk.git
cd emsdk
./emsdk install latest
./emsdk activate latest
source ./emsdk_env.sh
```

2. Build your project:
```bash
export EMCC_CFLAGS="-s FETCH=1"
cargo build --target wasm32-unknown-emscripten
```

3. Create an HTML file to load your WASM:
```html
<!DOCTYPE html>
<html>
<head>
    <title>Web Fetch Demo</title>
</head>
<body>
    <script src="target/wasm32-unknown-emscripten/debug/your_app.js"></script>
</body>
</html>
```

## Examples

See the `examples/` directory for more usage examples:

- `future_demo.rs` - Comprehensive demo of all features

Run examples:
```bash
cargo build --target wasm32-unknown-emscripten --example future_demo
python3 -m http.server 8000
# Open http://localhost:8000/future-demo.html
```

## API Reference

### `WebFetch::get(url: &str) -> FetchFuture`
Performs a GET request to the specified URL.

### `WebFetch::post(url: &str, body: &str) -> FetchFuture`
Performs a POST request with the given body.

### `WebFetch::fetch(url: &str, options: Option<FetchOptions>) -> FetchFuture`
Performs a custom HTTP request with full control over method, headers, body, and timeout.

### `FetchOptions`
```rust
pub struct FetchOptions {
    pub method: String,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
    pub timeout_ms: Option<u32>,
}
```

### `FetchResponse`
```rust
pub struct FetchResponse {
    pub status: u16,
    pub status_text: String,
    pub data: Vec<u8>,
    pub url: String,
}
```

## Architecture

The library uses Emscripten's Fetch API under the hood with a Rust-friendly async interface:

1. **FFI Layer**: Direct bindings to Emscripten's fetch functions
2. **Future Implementation**: Converts callbacks to Rust Futures using channels
3. **Runtime**: Custom executor optimized for browser event loop
4. **Type Safety**: Strong typing throughout the API

## Performance

- **Low Overhead**: Minimal abstraction over native Emscripten APIs
- **Efficient Polling**: Only active tasks are polled (not all tasks)
- **Memory Reuse**: Task slots are recycled to reduce allocations
- **Fast Wake**: 16ms maximum latency for task wake-up

## Requirements

- Rust 1.70+
- Emscripten SDK
- Target: `wasm32-unknown-emscripten`

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.