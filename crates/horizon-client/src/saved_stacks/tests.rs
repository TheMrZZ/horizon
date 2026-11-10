use super::*;
use horizon_world::{loot::PackOverlay, nbt::Tag};

#[test]
fn captured_roll_preserves_counts_slots_and_passive_nested_templates() {
    let root = std::env::temp_dir().join(format!("h088-roll-{}", std::process::id()));
    std::fs::create_dir_all(root.join("data/minecraft/loot_table")).unwrap();
    std::fs::write(
        root.join("items.json"),
        r#"{"minecraft:stone":{"minecraft:max_stack_size":64},"audit:missing":{}}"#,
    )
    .unwrap();
    let items = Items::catalog(&root, &PackOverlay::NONE).unwrap();
    let nested = Tag::List(vec![Tag::compound([
        ("id", Tag::String("minecraft:stone".into())),
        (
            "components",
            Tag::compound([("minecraft:max_stack_size", Tag::Int(64))]),
        ),
    ])]);
    let stack = |id: &str, size: Tag| {
        Tag::compound([
            ("Slot", Tag::Byte(5)),
            ("id", Tag::String(id.into())),
            // Do not route this witness through the unrelated count decoder.
            ("count", Tag::Int(120)),
            ("keep", Tag::Long(893)),
            (
                "components",
                Tag::compound([
                    ("minecraft:max_stack_size", size),
                    ("minecraft:bundle_contents", nested.clone()),
                ]),
            ),
        ])
    };
    for (id, size, elided) in [
        ("minecraft:stone", Tag::Int(64), true),
        ("minecraft:stone", Tag::Int(7), false),
        ("minecraft:stone", Tag::String("64".into()), false),
        ("audit:unknown", Tag::Int(64), false),
        ("audit:missing", Tag::Int(64), false),
    ] {
        let source = stack(id, size.clone());
        let mut roll = crate::chunk::RolledLoot {
            pos: (3, 200, 3),
            table: "minecraft:chests/simple_dungeon".into(),
            seed: Some(880),
            items: Tag::List(vec![source.clone()]),
        };
        capture_rolled_loot(&mut roll, &items);
        let Tag::List(saved) = &roll.items else {
            panic!("actual capture list")
        };
        let saved = &saved[0];
        for key in ["Slot", "id", "count", "keep"] {
            assert_eq!(saved.get(key), source.get(key));
        }
        assert_eq!(
            saved
                .get("components")
                .unwrap()
                .get("minecraft:bundle_contents"),
            Some(&nested)
        );
        assert_eq!(
            saved
                .get("components")
                .unwrap()
                .get("minecraft:max_stack_size"),
            (!elided).then_some(&size)
        );
        assert_eq!(roll.pos, (3, 200, 3));
        assert_eq!(roll.table, "minecraft:chests/simple_dungeon");
        assert_eq!(roll.seed, Some(880));
    }
    std::fs::remove_dir_all(root).unwrap();
}
