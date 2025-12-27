# Components Guide

Comprehensive guide to Sig's built-in components.

## TreeView

A high-performance tree view component with virtual scrolling, lazy loading, and search.

### Basic Usage

```rust
use sig::{TreeView, TreeNode};

// 1. Define your data type
#[derive(Clone, PartialEq, Eq, Hash)]
struct MyNode {
    id: usize,
    name: String,
    children: Vec<MyNode>,
}

// 2. Implement TreeNode trait
impl TreeNode for MyNode {
    fn label(&self) -> String {
        self.name.clone()
    }
    
    fn children(&self) -> Vec<Self> {
        self.children.clone()
    }
    
    fn has_children(&self) -> bool {
        !self.children.is_empty()
    }
}

// 3. Create TreeView
let root = MyNode { /* ... */ };

TreeView::new(root)
    .enable_search()
    .show_child_count(true)
    .item_height(28.0)
    .build()
```

### With Lazy Loading

Perfect for large data sets that need on-demand loading:

```rust
impl TreeNode for MyNode {
    fn load_children(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Load or parse children on demand
        parse_node(&self.data)?;
        Ok(())
    }
}
```

### Custom Rendering

```rust
TreeView::new(root)
    .build_with(Some(|node, ctx| {
        view()
            .style(move |s| {
                s.padding_left(ctx.depth as f32 * 20.0)
                 .background(if ctx.is_selected { 
                     HIGHLIGHT_COLOR 
                 } else { 
                     Color::TRANSPARENT 
                 })
            })
            .child(text(node.label()))
    }))
```

### Configuration Options

```rust
TreeView::new(root)
    .item_height(32.0)           // Row height (default: 28.0)
    .indent_width(20.0)          // Indentation per level (default: 16.0)
    .show_child_count(true)      // Show "(3)" after node name
    .enable_search()             // Add search box
    .disable_virtual_scroll()    // For small trees
    .build()
```

### Accessing State

```rust
let tree = TreeView::new(root);

// Get selected node signal
let selected = tree.selected();
create_effect(move || {
    if let Some(node) = selected.read().as_ref() {
        println!("Selected: {}", node.label());
    }
});

// Get search text signal
let search = tree.search_text();

tree.build()
```

### Key Features

- ✅ **Virtual scrolling** - Handles 10,000+ nodes efficiently
- ✅ **Lazy loading** - Load children on expand
- ✅ **Search/filter** - Built-in search functionality
- ✅ **Scroll preservation** - Position maintained on expand/collapse
- ✅ **Responsive icons** - Auto-updates on state changes
- ✅ **Customizable** - Full control over rendering

### TreeNode Requirements

**Important**: Use `Hash` for stable node identification:

```rust
pub trait TreeNode: Clone + PartialEq + Eq + Hash + 'static {
    fn label(&self) -> String;
    fn children(&self) -> Vec<Self>;
    fn has_children(&self) -> bool;
    fn load_children(&self) -> Result<(), Box<dyn std::error::Error>>;
}
```

**Why Hash?** Pointer addresses change when nodes are cloned. Hash provides stable identity.

### Wrapper Pattern for External Types

```rust
// Can't implement TreeNode for Arc<RwLock<Data>> (orphan rules)
// Solution: Create a wrapper

#[derive(Clone)]
struct DataTreeNode(Arc<RwLock<Data>>);

impl PartialEq for DataTreeNode {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for DataTreeNode {}

impl Hash for DataTreeNode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (Arc::as_ptr(&self.0) as usize).hash(state);
    }
}

impl TreeNode for DataTreeNode {
    // ... implementation
}
```

## VirtualList

High-performance list with virtual scrolling - only renders visible items.

### Basic Usage

```rust
let items = Signal::new((0..10000).collect::<Vec<_>>());

VirtualList::new(items)
    .item_height(40.0)
    .buffer_size(10)  // Extra items to render above/below
    .build(|item, index| {
        view()
            .style(|s| s.padding(10.0))
            .child(text(format!("Item {}: {}", index, item)))
    })
```

### With Custom Height

```rust
VirtualList::new(items)
    .height(500.0)       // Fixed container height
    .item_height(40.0)   // Each item height
    .build(|item, _| render_item(item))
```

### Important: Share the Signal

```rust
// ✅ Correct - signal shared, scroll preserved
let items = Signal::new(vec![]);
VirtualList::new(items.clone()).build(...)

// Later, update data
*items.write() = new_data;  // Scroll position preserved!

// ❌ Wrong - creates new list, loses scroll
dynamic(move || {
    VirtualList::new(Signal::new(items.read().clone())).build(...)
})
```

### Performance

- Renders only visible items + buffer
- 3-5x faster than full rendering for 1000+ items
- Uses smart diffing with `each` internally
- Scroll position preserved on data updates

## Resizable

Create resizable panels with drag handles.

### Horizontal Resize

```rust
let width = Signal::new(300.0);

view()
    .style(|s| s.flex().flex_row())
    .child((
        // Left panel (resizable)
        Resizable::horizontal(width.clone())
            .min_size(200.0)
            .max_size(600.0)
            .child(
                view()
                    .style(|s| s.w_full().h_full())  // Fill the panel
                    .child(/* content */)
            ),
        
        // Right panel (flexible)
        view()
            .style(|s| s.flex_grow(1.0).h_full())
            .child(/* content */),
    ))
```

### Vertical Resize

```rust
let height = Signal::new(200.0);

Resizable::vertical(height)
    .min_size(100.0)
    .max_size(400.0)
    .child(/* content */)
```

### Accessing Current Size

```rust
let size = Signal::new(300.0);

Resizable::horizontal(size.clone())
    .child(/* ... */);

// Read current size
create_effect(move || {
    println!("Current width: {}", size.read());
});
```

## Button

Interactive button component with variants.

### Basic Button

```rust
Button::new("Click me")
    .on_click(|_| println!("Clicked!"))
```

### With State

```rust
let count = Signal::new(0);

Button::new("Increment")
    .on_click(move |_| {
        *count.write() += 1;
    })
```

### Custom Styling

```rust
Button::new("Custom")
    .on_click(|_| {})
    .style(|s| {
        s.padding(16.0)
            .background(Color::BLUE)
            .color(Color::WHITE)
    })
```

## TextInput

Text input with IME support.

### Basic Usage

```rust
let text = Signal::new(String::new());

text_input()
    .placeholder("Enter name")
    .width(300.0)
    .value(text.clone())
```

### Two-way Binding

```rust
let username = Signal::new(String::new());

view()
    .child((
        text_input()
            .placeholder("Username")
            .value(username.clone()),
        
        // Live preview
        dynamic(move || {
            text(format!("Hello, {}!", username.read()))
        }),
    ))
```

### Multiline

```rust
text_input()
    .placeholder("Enter description")
    .width(400.0)
    .height(150.0)
    .value(description)
```

## Dynamic

Reactive content block - re-executes when tracked signals change.

### Basic Usage

```rust
let show = Signal::new(false);

dynamic(move || {
    if *show.read() {
        text("Visible")
    } else {
        text("Hidden")
    }
})
```

### With Multiple Children

```rust
dynamic(move || {
    if *loading.read() {
        fragment((
            text("Loading..."),
            spinner(),
        ))
    } else {
        content_view()
    }
})
```

## Fragment

Wrapper for multiple children without a parent container.

### Usage

```rust
// Instead of wrapping in a view
view().child((
    text("Title"),
    text("Content"),
))

// Use fragment to avoid extra layout node
fragment((
    text("Title"),
    text("Content"),
))
```

### In Conditionals

```rust
dynamic(move || {
    if *show_all.read() {
        fragment((
            header(),
            content(),
            footer(),
        ))
    } else {
        fragment(content())
    }
})
```

## Collapsible

Expandable/collapsible section.

### Basic Usage

```rust
let is_open = Signal::new(false);

Collapsible::new(is_open.clone())
    .header(text("Click to expand"))
    .content(
        view()
            .child(text("Hidden content"))
    )
```

## Separator

Visual separator line.

### Horizontal

```rust
Separator::horizontal()
    .style(|s| s.margin_y(16.0))
```

### Vertical

```rust
Separator::vertical()
    .style(|s| s.margin_x(16.0))
```

## Image

Display images from various sources.

### From Dynamic Image

```rust
use image::DynamicImage;

let img = Image::from_dynamic_image(dynamic_img);

img.size(300.0, 200.0)
    .style(|s| s.border_radius(8.0))
```

### From File (requires loading separately)

```rust
let img_data = image::open("path/to/image.png")?;
let img = Image::from_dynamic_image(img_data);

view().child(img.size(400.0, 300.0))
```

## Component Composition

Combine components to create complex UIs:

```rust
fn file_explorer(root: FileNode) -> View {
    let selected = Signal::new(None);
    
    view()
        .style(|s| s.flex().flex_row().h_full())
        .child((
            // Sidebar with tree
            TreeView::new(root)
                .enable_search()
                .build(),
            
            // Content area
            dynamic(move || {
                if let Some(file) = selected.read().as_ref() {
                    file_details(file)
                } else {
                    empty_state()
                }
            }),
        ))
}
```
