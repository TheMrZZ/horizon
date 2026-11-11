use super::*;
#[test]
#[ignore = "prepared resources; H088 actual registered entity-root save dispatch"]
fn registered_saved_stack_roots_use_the_same_loaded_catalog() {
    with_game(|g| {
        for (n, kind, key, id, size) in [
            (1, "item_frame", "Item", "minecraft:stone", 64),
            (2, "arrow", "item", "minecraft:arrow", 64),
            (3, "snowball", "Item", "minecraft:snowball", 16),
            (4, "small_fireball", "Item", "minecraft:stone", 64),
            (
                5,
                "firework_rocket",
                "FireworksItem",
                "minecraft:firework_rocket",
                64,
            ),
            (6, "trident", "item", "minecraft:trident", 1),
            (7, "item_display", "item", "minecraft:stone", 64),
            (8, "ominous_item_spawner", "item", "minecraft:stone", 64),
        ] {
            assert_eq!(g.items().get(id).unwrap().max_stack, size);
            let input = Tag::compound([
                ("UUID", Tag::IntArray(vec![88, 0, 0, n])),
                ("NoGravity", Tag::Byte(1)),
                (
                    key,
                    Tag::compound([
                        ("id", Tag::String(id.into())),
                        ("count", Tag::Int(1)),
                        (
                            "components",
                            Tag::compound([
                                ("minecraft:max_stack_size", Tag::Int(size as i32)),
                                (
                                    "minecraft:custom_data",
                                    Tag::compound([("keep", Tag::Int(889))]),
                                ),
                            ]),
                        ),
                    ]),
                ),
            ]);
            let command = format!(
                "summon minecraft:{kind} 8 200 8 {}",
                horizon_world::snbt_text::to_snbt(&input)
            );
            println!("H088 root setup={command}");
            assert!(g.run_command(&command), "registered root {kind}");
            let uuid = horizon_client::lan::playerdata::ints_uuid([88, 0, 0, n]);
            let target = g.target_by_uuid(uuid).expect("real loaded UUID");
            let tag = g.entity_nbt(target).expect("real save dispatch");
            println!("H088 {kind} full typed={tag:?} clock={}", g.world.game_time);
            assert_eq!(tag.get("UUID"), Some(&Tag::IntArray(vec![88, 0, 0, n])));
            assert_eq!(
                tag.get("id"),
                Some(&Tag::String(format!("minecraft:{kind}").as_str().into()))
            );
            let stack = tag.get(key).expect("typed actual stack field");
            assert_patch(stack, id, 1, None);
            assert!(
                stack
                    .get("components")
                    .unwrap()
                    .get("minecraft:custom_data")
                    .is_some()
            );
        }
    });
}
