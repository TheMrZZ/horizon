use super::*;
use crate::item_nbt::{item_from_nbt, item_to_nbt};
use horizon_world::{
    loot::PackOverlay,
    nbt::{Tag, parse_snbt},
};

#[test]
fn actual_capture_obeys_catalog_and_preserves_raw_nested_templates() {
    let root = std::env::temp_dir().join(format!("h088-capture-{}", std::process::id()));
    std::fs::create_dir_all(root.join("data/minecraft/loot_table")).unwrap();
    std::fs::write(root.join("items.json"), r#"{"minecraft:stone":{"minecraft:max_stack_size":64},"minecraft:ender_pearl":{"minecraft:max_stack_size":16},"minecraft:iron_helmet":{"minecraft:max_stack_size":1},"audit:missing":{}}"#).unwrap();
    let items = Items::catalog(&root, &PackOverlay::NONE).unwrap();
    for (id, size, elided) in [
        ("minecraft:stone", 64, true),
        ("minecraft:ender_pearl", 16, true),
        ("minecraft:iron_helmet", 1, true),
        ("audit:unknown", 64, false),
        ("audit:missing", 64, false),
        ("minecraft:stone", 7, false),
    ] {
        let source = parse_snbt(&format!(r#"{{id:"{id}",count:2,components:{{"minecraft:max_stack_size":{size},"minecraft:custom_data":{{keep:88}},"minecraft:bundle_contents":[{{id:"minecraft:stone",count:1,components:{{"minecraft:max_stack_size":64}}}}]}}}}"#)).unwrap().0;
        let live = item_from_nbt(&source).unwrap();
        let before = live.clone();
        let saved = capture(&live, &items);
        let encoded = item_to_nbt(&saved);
        assert_eq!(
            encoded
                .get("components")
                .unwrap()
                .get("minecraft:max_stack_size")
                .is_none(),
            elided,
            "{id}/{size}"
        );
        assert_eq!(live, before);
        assert_eq!(saved.item, live.item);
        assert_eq!(saved.count, 2);
        for key in ["minecraft:custom_data", "minecraft:bundle_contents"] {
            assert_eq!(
                encoded.get("components").unwrap().get(key),
                item_to_nbt(&live).get("components").unwrap().get(key)
            );
        }
        assert_eq!(capture(&saved, &items), saved);
    }
    let removal = item_from_nbt(
        &parse_snbt(
            r#"{id:"minecraft:stone",count:1,components:{"!minecraft:max_stack_size":{}}}"#,
        )
        .unwrap()
        .0,
    )
    .unwrap();
    assert_eq!(
        item_to_nbt(&capture(&removal, &items)),
        item_to_nbt(&removal)
    );
    // Same item ID, newly selected catalog: no static/default cache.
    std::fs::write(
        root.join("items.json"),
        r#"{"minecraft:stone":{"minecraft:max_stack_size":7}}"#,
    )
    .unwrap();
    let changed = Items::catalog(&root, &PackOverlay::NONE).unwrap();
    let mut stack = ItemStack::new("minecraft:stone", 2);
    stack
        .data_mut()
        .set_other("minecraft:max_stack_size", Some(Tag::Int(7)));
    assert!(capture(&stack, &changed).data().is_none());
    assert!(capture(&stack, &items).data().is_some());
    // Test-owned files only.
    std::fs::remove_dir_all(root).unwrap();
}
