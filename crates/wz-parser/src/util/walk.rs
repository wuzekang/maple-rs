use crate::{WzNodeArc, WzObjectType};

/// recursively walk a wz node, passing `&WzNodeArc` to `f`.
/// with `force_parse` it will parse every node along the way,
/// and only unparse `WzImage` after `f` is called to release memory.
pub fn walk_node(node: &WzNodeArc, force_parse: bool, f: &dyn Fn(&WzNodeArc)) {
    if force_parse {
        node.write().unwrap().parse(node).unwrap();
    }

    f(node);

    for child in node.read().unwrap().children.values() {
        walk_node(child, force_parse, f);
    }

    let is_wz_image = matches!(node.read().unwrap().object_type, WzObjectType::Image(_));

    if force_parse && is_wz_image {
        if let Ok(mut node) = node.write() {
            node.unparse();
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{property::WzSubProperty, WzNode, WzObjectType};

    fn generate_mock_node() -> WzNodeArc {
        let root = WzNode::from_str(
            "root",
            WzObjectType::Property(WzSubProperty::Property),
            None,
        )
        .into_lock();

        let child1 = WzNode::from_str("child1", 1, Some(&root)).into_lock();
        let child2 = WzNode::from_str("child2", 2, Some(&root)).into_lock();

        WzNode::from_str("child11", 11, Some(&child1)).into_lock();
        WzNode::from_str("child12", 12, Some(&child1)).into_lock();

        WzNode::from_str("child21", 21, Some(&child2)).into_lock();
        WzNode::from_str("child22", 22, Some(&child2)).into_lock();

        root
    }

    #[test]
    fn test_walk_node() {
        let root = generate_mock_node();

        let pathes = std::collections::HashSet::from([
            "root",
            "root/child1",
            "root/child1/child11",
            "root/child1/child12",
            "root/child2",
            "root/child2/child21",
            "root/child2/child22",
        ]);

        walk_node(&root, false, &|node| {
            let node_read = node.read().unwrap();
            assert!(pathes.contains(node_read.get_full_path().as_str()));
        });
    }
}
