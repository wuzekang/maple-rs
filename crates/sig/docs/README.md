# Sig Framework Documentation

Welcome to the Sig framework documentation!

## Quick Links

- **[../CLAUDE.md](../CLAUDE.md)** - Start here! Critical knowledge and quick reference
- **[COMPONENTS.md](COMPONENTS.md)** - Built-in components guide (TreeView, VirtualList, etc.)
- **[PATTERNS.md](PATTERNS.md)** - Advanced patterns and best practices
- **[TESTING.md](TESTING.md)** - Testing guidelines and patterns
- **[TROUBLESHOOTING.md](TROUBLESHOOTING.md)** - Common issues and solutions

## Documentation Structure

### CLAUDE.md (Critical Knowledge)
The essential guide that every developer should read first:
- The 5 most critical concepts
- Quick reference for common tasks
- Architecture overview
- Essential patterns
- Common mistakes checklist

**Read this first!**

### COMPONENTS.md (Component Reference)
Detailed documentation for all built-in components:
- TreeView - Tree structures with lazy loading
- VirtualList - High-performance lists
- Resizable - Resizable panels
- Button, TextInput, etc.
- Usage examples and configuration options

### PATTERNS.md (Best Practices)
Advanced design patterns:
- Reactivity patterns
- Component lifecycle management
- State management techniques
- Layout patterns
- Performance optimization
- Anti-patterns to avoid

### TESTING.md (Testing Guide)
How to write effective tests:
- Test setup and structure
- Testing reactive components
- Layout testing
- Common test utilities
- Complete test suite examples

### TROUBLESHOOTING.md (Problem Solving)
Quick solutions to common problems:
- Layout issues (background not filling, content collapsed, etc.)
- Reactivity issues (UI not updating, icons not changing, etc.)
- Component state issues (scroll position resets, toggle not working, etc.)
- Trait implementation issues (orphan rules, missing traits, etc.)
- Performance issues
- Quick fixes table

## Learning Path

### For Beginners

1. Read **CLAUDE.md** - Understand the critical concepts
2. Run examples in `../examples/` - See patterns in action
3. Read **COMPONENTS.md** - Learn available components
4. Read **TROUBLESHOOTING.md** - Know common pitfalls

### For Component Developers

1. Read **PATTERNS.md** - Learn advanced patterns
2. Read **TESTING.md** - Write good tests
3. Study existing components in `../src/components/`
4. Follow the component development checklist in CLAUDE.md

### For Debugging

1. Check **TROUBLESHOOTING.md** for your specific issue
2. Use the diagnostic checklist
3. Review relevant patterns in **PATTERNS.md**
4. Check examples for working code

## Examples

The `../examples/` directory contains working examples:

- `01_basics.rs` - Basic view creation and styling
- `02_dynamic.rs` - Reactive content with signals
- `05_render.rs` - Custom rendering
- `08_button_interactive.rs` - Interactive components
- `10_scroll_view.rs` - Scroll containers
- `11_text_input.rs` - Text input and IME

Run with:
```bash
cargo run -p sig --example 01_basics
```

## Tests

The `../tests/` directory contains comprehensive tests:

- `test_tree_view.rs` - TreeView component tests
- `test_signal_tracking.rs` - Signal reactivity tests
- `test_dynamic_cleanup.rs` - Scope cleanup tests
- `layout_*.rs` - Layout behavior tests

Run with:
```bash
cargo test -p sig
cargo test -p sig test_tree_view -- --nocapture
```

## Contributing

When adding new components or features:

1. Follow patterns in **PATTERNS.md**
2. Add tests following **TESTING.md**
3. Document in **COMPONENTS.md**
4. Add common issues to **TROUBLESHOOTING.md**
5. Update **CLAUDE.md** if adding critical concepts

## Quick Start

```rust
use sig::prelude::*;

fn main() -> anyhow::Result<()> {
    run(AppConfig::default(), app_view)
}

fn app_view() -> View {
    let count = Signal::new(0);
    
    view()
        .style(|s| {
            s.w_full().h_full()
                .flex().flex_col()
                .items_center().justify_center()
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

## Need Help?

1. **Check TROUBLESHOOTING.md** - Most common issues are covered
2. **Review examples** - See working code patterns
3. **Run tests** - Learn from test patterns
4. **Read source** - Components are well-documented

Happy coding! 🚀
