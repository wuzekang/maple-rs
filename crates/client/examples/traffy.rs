use taffy::prelude::*;

fn main() {
    // First create an instance of TaffyTree
    let mut tree: TaffyTree<()> = TaffyTree::new();

    // Create a tree of nodes using `TaffyTree.new_leaf` and `TaffyTree.new_with_children`.
    // These functions both return a node id which can be used to refer to that node
    // The Style struct is used to specify styling information
    let header_node = tree
        .new_leaf(Style {
            size: Size {
                width: length(800.0),
                height: length(100.0),
            },
            ..Default::default()
        })
        .unwrap();

    let body_node = tree
        .new_leaf(Style {
            size: Size {
                width: length(800.0),
                height: auto(),
            },
            flex_grow: 1.0,
            ..Default::default()
        })
        .unwrap();

    let root_node = tree
        .new_with_children(
            Style {
                flex_direction: FlexDirection::Column,
                size: Size {
                    width: length(800.0),
                    height: length(600.0),
                },
                ..Default::default()
            },
            &[header_node, body_node],
        )
        .unwrap();

    // Call compute_layout on the root of your tree to run the layout algorithm
    tree.compute_layout(root_node, Size::MAX_CONTENT).unwrap();

    // Inspect the computed layout using `TaffyTree.layout`
    assert_eq!(tree.layout(root_node).unwrap().size.width, 800.0);
    assert_eq!(tree.layout(root_node).unwrap().size.height, 600.0);
    assert_eq!(tree.layout(header_node).unwrap().size.width, 800.0);
    assert_eq!(tree.layout(header_node).unwrap().size.height, 100.0);
    assert_eq!(tree.layout(body_node).unwrap().size.width, 800.0);
    assert_eq!(tree.layout(body_node).unwrap().size.height, 500.0); // This value was not set explicitly, but was computed by Taffy
}
