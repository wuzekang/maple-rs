# Maple RS - GMS 083

- An experimental project exploring the reconstruction of GMS 083 client using modern Rust technology stack.
- Currently in proof-of-concept stage, only basic framework and core module prototypes are implemented, not yet playable.

## 🚀 Quick Start


### 1. Clone Repository
```bash
git clone https://github.com/wuzekang/maple-rs.git --depth 1
```

### 2. Download Resources

- QQ Group: 1042028998
- Download data.zip from group shared files and extract to maple-rs/data directory

### 3. Build and Run
```bash
cd maple-rs

# Split WZ files into individual IMG files for on-demand loading
cargo run --bin wz-splitter split --output ./data ./data/Base.wz

cargo run --bin client
cargo run --bin editor
```

## 🖥️ Running Screenshots
 
### crates/client
![](./client-login.png)
![](./client-main.png)

### crates/editor
![](./editor.png)

## ⚙️ Tech Stack

- **Layout**: taffy
- **Graphics Rendering**: sdl3-sys
- **Text Rendering**: cosmic-text

## 🏗️ Project Structure

```text
crates
├── client              # Main client implementation (game loop core)
│   ├── app             # Root component (login/character selection/map switching)
│   ├── scene           # Map scene rendering
│   ├── map             # Map loading
│   └── wz              # Resource loading
├── editor              # WZ structure preview
├── reactive            # Reactive core (Solid-like)
├── ui                  # Reactive UI framework
│   ├── render          # Rendering abstraction layer
│   │   └── command.rs  # Drawing commands
│   ├── widget          # Component-based UI system
│   │   ├── debug       # DevTools (element inspection/FPS panel)
│   │   ├── dynamic     # Dynamic content renderer (supports conditional rendering/async loading)
│   │   ├── focus_trap  # Focus management
│   │   ├── text        # Text rendering
│   │   ├── text_input  # Text input component
│   │   ├── dynamic     # Dynamic content renderer (supports conditional rendering/async loading)
│   │   └── scroll_view # Scroll container
│   └── geometry/       # Mathematical foundation library (including Bézier curve calculations)
└── wz-reader-rs        # WZ parser library
```

## 🤝 Contribution Guide

Welcome to participate through Issues and PRs in the following areas:
- New feature development
- Performance optimization
- Platform compatibility improvements
- Documentation translation

## 📜 Disclaimer

This project is developed for learning and research purposes only, and does not contain any game resource files. Game copyrights belong to NEXON. Please obtain the game client through legal channels.