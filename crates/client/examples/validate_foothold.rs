use client::wz::{self, Node};
use indexmap::IndexMap as HashMap;

fn main() {
    let root = wz::resolve_base().unwrap();
    let maps = root.at_path("Map/Map").unwrap().children();
    let a: Vec<
        HashMap<String, HashMap<String, HashMap<String, HashMap<String, HashMap<String, i32>>>>>,
    > = maps
        .into_iter()
        .filter(|(name, _)| name.to_string().starts_with("Map"))
        .map(|(_, node)| {
            node.children()
                .into_iter()
                .take(10)
                .filter_map(|(name, node)| {
                    Some((name.to_string(), (node.parse().try_get("foothold")?.into())))
                })
                .collect()
        })
        .collect();

    println!("{:?}", a.len());

    for item in a {
        for (l1, node) in item {
            for (l2, node) in node {
                for (l3, node) in node {
                    let keys: Vec<&String> = node.keys().collect();
                    println!("{:?}", keys);
                    // foothold 连续 fail
                    // for (prev, next) in keys.iter().skip(1).zip(keys.iter().take(keys.len() - 1)) {
                    //     if prev.parse::<i32>().unwrap() != next.parse::<i32>().unwrap() + 1 {
                    //         panic!("err {l1} {l2} {l3} {prev} {next}");
                    //     }
                    // }

                    // foothold 只有一个 head
                    let heads = node
                        .iter()
                        .filter_map(|(k, v)| if v["prev"] == 0 { Some(k) } else { None })
                        .collect::<Vec<_>>();
                    let tails = node
                        .iter()
                        .filter_map(|(k, v)| if v["next"] == 0 { Some(k) } else { None })
                        .collect::<Vec<_>>();
                    if heads.len() != tails.len() {
                        panic!("err {l1} {l2} {l3} {heads:?} {tails:?}");
                    }
                }
            }
        }
    }

    // println!("{:?}", a);
}
