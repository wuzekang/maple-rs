use wz_parser::property::{Vector2D, WzValue};
use wz_parser::util::{self, node_util};
use wz_parser::version::WzMapleVersion;
use wz_parser::{node, wz_image, WzNode, WzNodeArc, WzNodeCast, WzObjectType};

type Error = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Error>;

/**
 * the test.wz file structure:
 *   test
 *     - wz_img.img
 *       - conv
 *         - 0(vec)
 *         - 1(png)
 *           - origin
 *           - _inlink
 *       - 1
 *         - float
 *         - double
 *         - long
 *         - short
 *         - int
 *       - 2
 *         - nil
 *         - string
 *         - uol
 *     - wz_dir
 *       - wz_img_under_dir.img
 *         - hi
 */

fn make_sure_wz_file_version(node: &WzNodeArc, version: i32) {
    let wz_file_read = node.read().unwrap();

    assert!(matches!(wz_file_read.object_type, WzObjectType::File(_)));

    if let WzObjectType::File(file) = &wz_file_read.object_type {
        assert_eq!(file.wz_file_meta.patch_version, version);
    }
}

#[test]
fn should_guessing_patch_version() -> Result<()> {
    let wz_file = WzNode::from_wz_file_full(
        r"tests/test.wz",
        Some(WzMapleVersion::BMS),
        None,
        None,
        None,
    );
    assert!(wz_file.is_ok());

    let wz_file = wz_file?.into_lock();

    make_sure_wz_file_version(&wz_file, -1);

    assert!(node_util::parse_node(&wz_file).is_ok());

    make_sure_wz_file_version(&wz_file, 123);

    Ok(())
}

#[test]
fn should_parsing_with_patch_version() -> Result<()> {
    let wz_file = WzNode::from_wz_file_full(
        r"tests/test.wz",
        Some(WzMapleVersion::BMS),
        Some(123),
        None,
        None,
    );
    assert!(wz_file.is_ok());

    let wz_file = wz_file?.into_lock();

    assert!(node_util::parse_node(&wz_file).is_ok());

    make_sure_wz_file_version(&wz_file, 123);

    Ok(())
}

fn check_sample_wz_dir(wz_dir: &WzNodeArc) -> Result<()> {
    let wz_dir = wz_dir.read().unwrap();

    let sub_img = wz_dir.at("wz_img_under_dir.img");
    assert!(sub_img.is_some());

    let sub_img = sub_img.unwrap();

    assert!(sub_img.read().unwrap().try_as_image().is_some());

    assert!(node_util::parse_node(&sub_img).is_ok());

    let child = sub_img.read().unwrap().at("hi");
    assert!(child.is_some());

    let child = child.unwrap();
    let child = child.read().unwrap();

    assert!(matches!(child.object_type, WzObjectType::Value(_)));

    if let Some(num) = child.try_as_int() {
        assert_eq!(num, &1);
    }

    Ok(())
}

fn check_sample_wz_img(wz_img: &WzNodeArc) -> Result<()> {
    assert!(node_util::parse_node(wz_img).is_ok());

    let wz_img_read = wz_img.read().unwrap();

    if let Some(first_folder) = wz_img_read.at("1") {
        let first_folder = first_folder.read().unwrap();

        assert!(first_folder.is_sub_property());

        let int = first_folder.at("int");
        assert!(int.is_some());

        let int = int.unwrap();
        let int = int.read().unwrap();
        assert_eq!(int.try_as_int(), Some(&1));

        let short = first_folder.at("short");
        assert!(short.is_some());

        let short = short.unwrap();
        let short = short.read().unwrap();
        assert_eq!(short.try_as_short(), Some(&2));

        let long = first_folder.at("long");
        assert!(long.is_some());

        let long = long.unwrap();
        let long = long.read().unwrap();
        assert_eq!(long.try_as_long(), Some(&3));

        let float = first_folder.at("float");
        assert!(float.is_some());

        let float = float.unwrap();
        let float = float.read().unwrap();
        assert_eq!(float.try_as_float(), Some(&4.1));

        let double = first_folder.at("double");
        assert!(double.is_some());

        let double = double.unwrap();
        let double = double.read().unwrap();
        assert_eq!(double.try_as_double(), Some(&4.2));
    }

    if let Some(second_folder) = wz_img_read.at("2") {
        let second_folder = second_folder.read().unwrap();

        assert!(second_folder.is_sub_property());

        let nil = second_folder.at("nil");
        assert!(nil.is_some());

        let nil = nil.unwrap();
        let nil = nil.read().unwrap();
        assert!(nil.is_null());

        let string = second_folder.at("string");
        assert!(string.is_some());

        let string = string.unwrap();
        let string = string.read().unwrap();
        if let Some(string) = string.try_as_string() {
            let s = string.get_string();
            assert!(s.is_ok());
            assert_eq!(&s?, "foo");
        }

        let uol = second_folder.at("uol");
        assert!(uol.is_some());

        let uol = uol.unwrap();
        let uol = uol.read().unwrap();

        // uol node should be resolved
        assert!(uol.try_as_uol().is_none());

        if let Some(str) = uol.try_as_string() {
            let s = str.get_string();
            assert!(s.is_ok());
            assert_eq!(&s?, "foo");
        }
    }

    if let Some(convex_folder) = wz_img_read.at("conv") {
        let convex_folder = convex_folder.read().unwrap();

        assert!(convex_folder.is_convex());

        let vector = convex_folder.at("0");
        assert!(vector.is_some());

        let vector = vector.unwrap();
        if let Some(vec) = vector.read().unwrap().try_as_vector2d() {
            assert_eq!(vec, &Vector2D(1, 1));
        }

        let png = convex_folder.at("1");
        assert!(png.is_some());

        let png = png.unwrap();
        let png = png.read().unwrap();
        assert!(png.try_as_png().is_some());

        let vector = png.at("origin");
        assert!(vector.is_some());

        let vector = vector.unwrap();
        if let Some(vec) = vector.read().unwrap().try_as_vector2d() {
            assert_eq!(vec, &Vector2D(0, 0));
        }

        let inlink = png.at("_inlink");
        assert!(inlink.is_some());

        let inlink = inlink.unwrap();
        let inlink_str = if let Some(string) = inlink.read().unwrap().try_as_string() {
            string.get_string()?
        } else {
            String::new()
        };
        assert_eq!(&inlink_str, "conv/png2");
    }

    Ok(())
}

#[test]
fn should_parsing_wz_file_and_check_values() -> Result<()> {
    let wz_file = WzNode::from_wz_file_full(
        r"tests/test.wz",
        Some(WzMapleVersion::BMS),
        Some(123),
        None,
        None,
    );
    assert!(wz_file.is_ok());

    let wz_file = wz_file?.into_lock();

    assert!(node_util::parse_node(&wz_file).is_ok());

    let wz_file_read = wz_file.read().unwrap();

    let wz_dir = wz_file_read.at("wz_dir");
    assert!(wz_dir.is_some());

    assert!(check_sample_wz_dir(&wz_dir.unwrap()).is_ok());

    let wz_img = wz_file_read.at("wz_img.img");
    assert!(wz_img.is_some());

    assert!(check_sample_wz_img(&wz_img.unwrap()).is_ok());

    Ok(())
}

#[test]
fn should_success_using_wz_node_methods_on_childs() -> Result<()> {
    let wz_file = WzNode::from_wz_file_full(
        r"tests/test.wz",
        Some(WzMapleVersion::BMS),
        Some(123),
        None,
        None,
    );
    assert!(wz_file.is_ok());

    let wz_file = wz_file?.into_lock();

    assert!(node_util::parse_node(&wz_file).is_ok());

    let wz_file_read = wz_file.read().unwrap();

    let wz_img = wz_file_read.at("wz_img.img");
    assert!(wz_img.is_some());

    let wz_node_not_exist = wz_file_read.at("not_exist");
    assert!(wz_node_not_exist.is_none());

    let wz_img = wz_img.unwrap();
    assert!(node_util::parse_node(&wz_img).is_ok());

    let nil_node = wz_file_read.at_path("wz_img.img/2/nil");
    assert!(nil_node.is_some());

    let wz_node_not_exist = wz_file_read.at_path("wz_img.img/2/not_exist");
    assert!(wz_node_not_exist.is_none());

    let nil_node = nil_node.unwrap();
    let nil_read = nil_node.read().unwrap();
    assert_eq!(nil_read.get_full_path(), "test/wz_img.img/2/nil");

    let nil_parent = nil_read.at_relative("..");
    assert!(nil_parent.is_some());
    assert_eq!(
        nil_parent.unwrap().read().unwrap().get_full_path(),
        "test/wz_img.img/2"
    );

    let vec_node = nil_read.at_path_relative("../../conv/0");
    assert!(vec_node.is_some());
    let vec_node = vec_node.unwrap();
    assert_eq!(
        vec_node.read().unwrap().get_full_path(),
        "test/wz_img.img/conv/0"
    );

    let png_node = nil_read.at_path_relative("../../conv/1");
    assert!(png_node.is_some());
    let png_node = png_node.unwrap();
    assert_eq!(
        png_node.read().unwrap().get_full_path(),
        "test/wz_img.img/conv/1"
    );

    let wz_node_not_exist = nil_read.at_path_relative("../../not_exist");
    assert!(wz_node_not_exist.is_none());

    let inlink = png_node.read().unwrap().at("_inlink");
    assert!(inlink.is_some());
    let inlink = inlink.unwrap();
    let inlink_read = inlink.read().unwrap();
    let inlink_string = inlink_read.try_as_string();

    assert!(inlink_string.is_some());
    let inlink_string = inlink_string.unwrap().get_string();

    assert!(inlink_string.is_ok());

    let inlink_string = inlink_string.unwrap();
    let inlink_target = node_util::resolve_inlink(&inlink_string, &inlink);
    assert!(inlink_target.is_none());

    let parent_img = png_node.read().unwrap().get_parent_wz_image();
    assert!(parent_img.is_some());
    let parent_img = parent_img.unwrap();
    assert_eq!(
        parent_img.read().unwrap().get_full_path(),
        wz_img.read().unwrap().get_full_path()
    );

    let force_get_next_exist_node = parent_img.read().unwrap().at_path_parsed("2/not_exist");
    assert!(matches!(
        force_get_next_exist_node,
        Err(node::Error::NodeNotFound)
    ));

    let force_get_some_node = parent_img.read().unwrap().at_path_parsed("2/nil");
    assert!(force_get_some_node.is_ok());

    parent_img.write().unwrap().unparse();

    assert_eq!(parent_img.read().unwrap().children.len(), 0);

    let parent_img_read = parent_img.read().unwrap();
    if let WzObjectType::Image(wz_image) = &parent_img_read.object_type {
        let direct_access_not_exist = wz_image.at_path("2/not_exist");

        assert!(matches!(
            direct_access_not_exist,
            Err(wz_image::Error::ParsePropertyListError(
                util::WzPropertyParseError::NodeNotFound
            ))
        ));

        let direct_access_nil = wz_image.at_path("2/nil");
        assert!(direct_access_nil.is_ok());

        let nil = direct_access_nil.unwrap();
        let nil = nil.read().unwrap();
        assert!(matches!(
            nil.object_type,
            WzObjectType::Value(WzValue::Null)
        ));
    }

    Ok(())
}

#[test]
fn should_success_walk_thorugh() {
    let wz_file = WzNode::from_wz_file_full(
        r"tests/test.wz",
        Some(WzMapleVersion::BMS),
        Some(123),
        None,
        None,
    );
    assert!(wz_file.is_ok());

    let wz_file = wz_file.unwrap().into_lock();

    let pathes = std::collections::HashSet::from([
        "test",
        "test/wz_img.img",
        "test/wz_img.img/conv",
        "test/wz_img.img/conv/0",
        "test/wz_img.img/conv/1",
        "test/wz_img.img/conv/1/_inlink",
        "test/wz_img.img/conv/1/origin",
        "test/wz_img.img/1",
        "test/wz_img.img/1/float",
        "test/wz_img.img/1/double",
        "test/wz_img.img/1/long",
        "test/wz_img.img/1/short",
        "test/wz_img.img/1/int",
        "test/wz_img.img/2",
        "test/wz_img.img/2/nil",
        "test/wz_img.img/2/string",
        "test/wz_img.img/2/uol",
        "test/wz_dir",
        "test/wz_dir/wz_img_under_dir.img",
        "test/wz_dir/wz_img_under_dir.img/hi",
    ]);

    util::walk_node(&wz_file, true, &|node| {
        let node_read = node.read().unwrap();
        assert!(pathes.contains(node_read.get_full_path().as_str()));
    });
}

#[test]
fn should_guessing_iv() -> Result<()> {
    let wz_file = WzNode::from_wz_file(r"tests/test_need_iv.wz", None);
    assert!(wz_file.is_ok());

    let wz_file = wz_file?.into_lock();

    assert!(wz_file
        .read()
        .unwrap()
        .try_as_file()
        .map(|file| assert_eq!(file.reader.wz_iv, [0xB9, 0x7D, 0x63, 0xE9]))
        .is_some());

    make_sure_wz_file_version(&wz_file, -1);

    assert!(node_util::parse_node(&wz_file).is_ok());

    make_sure_wz_file_version(&wz_file, 123);

    Ok(())
}

#[test]
fn should_success_walk_thorugh_with_iv() {
    let wz_file = WzNode::from_wz_file_full(
        r"tests/test_need_iv.wz",
        Some(WzMapleVersion::EMS),
        Some(123),
        None,
        None,
    );
    assert!(wz_file.is_ok());

    let wz_file = wz_file.unwrap().into_lock();

    let pathes = std::collections::HashSet::from([
        "test_need_iv",
        "test_need_iv/wz_img.img",
        "test_need_iv/wz_img.img/conv",
        "test_need_iv/wz_img.img/conv/0",
        "test_need_iv/wz_img.img/conv/1",
        "test_need_iv/wz_img.img/conv/1/_inlink",
        "test_need_iv/wz_img.img/conv/1/origin",
        "test_need_iv/wz_img.img/1",
        "test_need_iv/wz_img.img/1/float",
        "test_need_iv/wz_img.img/1/double",
        "test_need_iv/wz_img.img/1/long",
        "test_need_iv/wz_img.img/1/short",
        "test_need_iv/wz_img.img/1/int",
        "test_need_iv/wz_img.img/2",
        "test_need_iv/wz_img.img/2/nil",
        "test_need_iv/wz_img.img/2/string",
        "test_need_iv/wz_img.img/2/uol",
        "test_need_iv/wz_dir",
        "test_need_iv/wz_dir/wz_img_under_dir.img",
        "test_need_iv/wz_dir/wz_img_under_dir.img/hi",
    ]);

    util::walk_node(&wz_file, true, &|node| {
        let node_read = node.read().unwrap();
        println!("{}", node_read.get_full_path());
        assert!(pathes.contains(node_read.get_full_path().as_str()));
    });
}
