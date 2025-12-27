//! Test dirty nodes optimization
//!
//! This example demonstrates the dirty nodes optimization
//! that prevents style dirty node explosion in VirtualList

use sig::*;

fn main() {
    create_scope(|| {
        let data = Signal::new((0..10000).collect::<Vec<_>>());

        app(
            virtual_list(data)
                .height(600.0)
                .item_height(40.0)
                .buffer_size(5)
                .build(|item, index| {
                    view()
                        .style(|s| {
                            s.padding(10.0)
                                .border_bottom(1.0)
                                .hover(|s| s.background(Color::rgb(240, 240, 240)))
                        })
                        .child(text(format!("Item #{}: {}", index, item)))
                }),
        )
        .style(|s| s.w_full().h_full())
        .run();
    });
}
