# AGENTS.md

This file provides essential guidance for AI coding assistants when working with the Sig framework.

## Table of Contents

- [About Sig](#about-sig)
- [Quick Start](#quick-start)
- [Core Concepts & Common Pitfalls](#core-concepts--common-pitfalls)
- [Essential Patterns](#essential-patterns)
- [Architecture Overview](#architecture-overview)
- [API Quick Reference](#api-quick-reference)
- [Build & Test Commands](#build--test-commands)
- [Development Checklist](#development-checklist)
- [Documentation](#documentation)

---

## About Sig

Sig is a reactive GUI framework for Rust inspired by SolidJS, featuring:

- **Fine-grained reactivity** - Signal-based state management with automatic dependency tracking
- **GPU-accelerated rendering** - Vello for high-performance 2D graphics
- **Flexbox layout** - Taffy for CSS-like layout
- **Advanced text rendering** - Parley for text layout and rendering

---

## Quick Start

### Minimal Application

```rust
use sig::prelude::*;

fn main() -> anyhow::Result<()> {
    run(AppConfig::default(), app_view)
}

fn app_view() -> View {
    let count = Signal::new(0);
    
    view()
        .style(|s| {
            s.w_full()
                .h_full()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .gap(16.0)
        })
        .child((
            dynamic(move || {
                text(format!("Count: {}", count.read()))
            }),
            Button::new("Increment")
                .on_click(move |_| {
                    *count.write() += 1;
                }),
        ))
}
```

### Key Takeaways

1. Use `Signal<T>` for reactive state
2. Wrap reactive reads in `dynamic()`
3. Views are flex containers by default
4. Use `.w_full()` and `.h_full()` for sizing

---

## Essential Patterns

### Application Structure

```rust
use sig::prelude::*;

fn main() -> anyhow::Result<()> {
    run(AppConfig::default(), || {
        view()
            .style(|s| s.w_full().h_full().flex().flex_col())
            .child(/* your content here */)
    })
}
```

### Signal Management

```rust
// Create signal
let count = Signal::new(0);

// Read (reactive - tracks dependency)
dynamic(move || {
    text(format!("Count: {}", count.read()))
})

// Read (non-reactive - no tracking)
let value = count.read_untracked();

// Write (notifies observers on drop)
*count.write() += 1;

// Effects (side effects that track dependencies)
create_effect(move || {
    println!("Count changed to: {}", count.read());
});
```

### Fixed Header + Scrollable Content

```rust
view()
    .style(|s| s.w_full().h_full().flex().flex_col())
    .child((
        // Header (fixed height)
        view()
            .style(|s| {
                s.w_full()
                    .padding(16.0)
                    .flex_shrink(0.0)  // Don't shrink
            }),
        
        // Content (scrollable, fills remaining space)
        view()
            .style(|s| {
                s.w_full()
                    .flex_grow(1.0)      // Take remaining space
                    .min_height(0.0)     // ← Critical for scrolling!
                    .overflow_y_scroll()
            }),
    ))
```

### Resizable Panels

```rust
let panel_width = Signal::new(300.0);

view()
    .style(|s| s.w_full().h_full().flex().flex_row())
    .child((
        // Left panel (user-resizable)
        Resizable::horizontal(panel_width)
            .min_size(200.0)
            .max_size(600.0)
            .child(/* left panel content */),
        
        // Right panel (fills remaining space)
        view()
            .style(|s| s.flex_grow(1.0).h_full())
            .child(/* right panel content */),
    ))
```

### Dynamic Lists

```rust
let items = Signal::new(vec!["Item 1", "Item 2", "Item 3"]);

// Simple list
dynamic(move || {
    view()
        .style(|s| s.flex().flex_col().gap(8.0))
        .child(
            items.read()
                .iter()
                .map(|item| text(*item))
                .collect::<Vec<_>>()
        )
})

// Virtualized list (better for large datasets)
VirtualList::new(items)
    .item_height(40.0)
    .build(|item, index| {
        text(format!("{}: {}", index, item))
    })
```

---

## Architecture Overview

### Core Modules

| Module | Path | Purpose |
|--------|------|---------|
| **Core** | `src/core/` | Component system (`View`, `Element` trait, `ViewTuple`) |
| **Reactive** | `src/reactive/` | State management (`Signal`, `create_effect`, `Scope`) |
| **Style** | `src/style/` | Styling system (`StyleBuilder`, Taffy + Render styles) |
| **Layout** | `src/layout/` | Flexbox layout (powered by Taffy) |
| **Event** | `src/event/` | Interaction (`Interactive` trait, event bubbling) |
| **Render** | `src/render/` | GPU rendering (Vello + Parley) |
| **Components** | `src/components/` | Built-in components (`Button`, `TextInput`, `VirtualList`, etc.) |

### Runtime Architecture

Sig uses **thread-local state domains** for different subsystems:

- `ReactiveState` - Signals and effects tracking
- `LayoutState` - Taffy layout tree
- `InputState` - Focus management and IME
- `WindowState` - Window lifecycle and redraw requests
- `RenderCtxState` - Vello rendering context

### Component Hierarchy

```
Element trait
  ├── View (main container component)
  ├── Dynamic (reactive content)
  ├── Text (text nodes)
  └── Custom components (Button, TextInput, etc.)
```

---

## API Quick Reference

### Layout Styles

```rust
// Size
.w_full()              // width: 100%
.h_full()              // height: 100%
.size_full()           // both
.width(px)             // fixed width
.height(px)            // fixed height
.min_width(px)         // minimum width
.min_height(px)        // minimum height
.max_width(px)         // maximum width
.max_height(px)        // maximum height

// Flex
.flex()                // display: flex
.flex_row()            // flex-direction: row (default)
.flex_col()            // flex-direction: column
.flex_grow(n)          // flex-grow: n
.flex_shrink(n)        // flex-shrink: n

// Spacing
.gap(px)               // gap between children
.padding(px)           // padding: all sides
.padding_x(px)         // horizontal padding
.padding_y(px)         // vertical padding
.margin(px)            // margin: all sides
.margin_x(px)          // horizontal margin
.margin_y(px)          // vertical margin

// Alignment
.items_center()        // align-items: center
.items_start()         // align-items: flex-start
.items_end()           // align-items: flex-end
.justify_center()      // justify-content: center
.justify_start()       // justify-content: flex-start
.justify_end()         // justify-content: flex-end
.justify_between()     // justify-content: space-between

// Overflow
.overflow_y_scroll()   // vertical scroll (overlay)
.overflow_x_scroll()   // horizontal scroll (overlay)
.overflow_scroll()     // both directions
```

### Visual Styles

```rust
// Colors
.background(Color)     // background color
.color(Color)          // text color
.border_color(Color)   // border color

// Borders
.border(width)         // all borders
.border_radius(px)     // rounded corners
.border_top(width)     // top border
.border_bottom(width)  // bottom border
.border_left(width)    // left border
.border_right(width)   // right border

// Opacity
.opacity(0.0..=1.0)    // element opacity
```

### Reactive APIs

```rust
// Signals
Signal::new(value)              // Create signal
signal.read()                   // Reactive read (tracks dependency)
signal.read_untracked()         // Non-reactive read
*signal.write() = value         // Write (notifies on drop)

// Effects
create_effect(|| { ... })       // Side effect with auto-tracking
dynamic(|| { ... })             // Reactive UI component

// Testing utilities
sig::runtime::flush_pending_signals()  // Process pending updates
```

### Built-in Components

```rust
// TreeView
TreeView::new(root_node)
    .enable_search()
    .show_child_count(true)
    .build()

// VirtualList (for large lists)
VirtualList::new(items_signal)
    .item_height(40.0)
    .build(|item, index| {
        text(format!("{}: {}", index, item))
    })

// Button
Button::new("Click me")
    .on_click(|_| println!("Clicked"))

// TextInput
text_input()
    .placeholder("Enter text...")
    .value(text_signal)

// Resizable panel
Resizable::horizontal(width_signal)
    .min_size(200.0)
    .max_size(600.0)
    .child(/* content */)
```

---

## Build & Test Commands

```bash
# Build the sig crate
cargo build -p sig

# Run examples
cargo run -p sig --example 01_basics
cargo run -p sig --example 02_counter
cargo run -p sig --example 03_tree_view

# Run all tests
cargo test -p sig

# Run specific test with output
cargo test -p sig test_tree_view -- --nocapture

# Build documentation
cargo doc -p sig --open
```

---

## Development Checklist

Use this checklist when building or reviewing Sig applications:

### Layout
- [ ] Added `w_full()` / `h_full()` for proper sizing
- [ ] Used `min_height(0)` with `flex_grow` in nested flex layouts
- [ ] Verified scrollable containers have proper overflow styles

### Reactivity
- [ ] Wrapped reactive state reads in `dynamic()`
- [ ] Created components once (not inside `dynamic`)
- [ ] Used `read_untracked()` where reactivity is not needed
- [ ] Called `flush_pending_signals()` in tests after state changes

### Performance
- [ ] Used `VirtualList` for large lists (100+ items)
- [ ] Avoided unnecessary clones in reactive contexts
- [ ] Used stable keys (Hash) for list items, not pointer addresses

### Events
- [ ] Attached event handlers to appropriate elements
- [ ] Handled cleanup in drop implementations if needed

---

## Documentation

### Core Documentation Files

- **[docs/COMPONENTS.md](docs/COMPONENTS.md)** - Detailed component API documentation
- **[docs/PATTERNS.md](docs/PATTERNS.md)** - Advanced patterns and best practices
- **[docs/TESTING.md](docs/TESTING.md)** - Testing guidelines and examples
- **[docs/TROUBLESHOOTING.md](docs/TROUBLESHOOTING.md)** - Common issues and solutions

### Learning Path

1. **Start here** - Read this AGENT.md for core concepts
2. **Try examples** - Run examples in `examples/` directory
3. **Build simple apps** - Create counter, todo list, etc.
4. **Advanced features** - Read PATTERNS.md for complex scenarios
5. **Contribute** - Read TROUBLESHOOTING.md for common pitfalls
