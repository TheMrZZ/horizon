use super::*;
#[test]
#[ignore = "prepared resources; H088 catalog snapshot and retained loot witness"]
fn captured_roots_keep_unchanged_loot_and_detect_real_mutations() {
    use crate::game::minecarts::{CartKind, Minecart};
    use horizon_client::save::{self, RolledLoot};
    with_game(|g| {
        assert!(g.run_command("item replace entity @s hotbar.0 with minecraft:stone[minecraft:max_stack_size=64,minecraft:custom_data={keep:892}] 2"));
        let stack = g.inventory[0].clone();
        let mut cart = Minecart::new(880, CartKind::Chest, Vec3::new(8.0, 200.0, 8.0));
        cart.items[5] = stack.clone();
        let roll = RolledLoot {
            pos: (3, 200, 3),
            table: "minecraft:chests/simple_dungeon".into(),
            seed: Some(880),
            items: save::encode_items(&cart.items),
        };
        cart.loot = Some(roll.clone());
        let saved = cart.to_nbt_with_items([88, 0, 0, 80], None, g.items());
        println!(
            "H088 unchanged rolled cart full typed={saved:?} clock={}",
            g.world.game_time
        );
        assert_eq!(
            saved.get("LootTable"),
            Some(&Tag::String(roll.table.as_str().into()))
        );
        assert_eq!(saved.get("LootTableSeed"), Some(&Tag::Long(880)));
        assert!(saved.get("Items").is_none());
        assert_eq!(cart.loot.as_ref(), Some(&roll));
        assert_eq!(cart.items[5], stack);
        cart.items[5].as_mut().unwrap().count = 1;
        let changed = cart.to_nbt_with_items([88, 0, 0, 80], None, g.items());
        println!("H088 changed rolled cart full typed={changed:?}");
        assert!(changed.get("LootTable").is_none());
        let Tag::List(items) = changed.get("Items").unwrap() else {
            panic!("Items")
        };
        assert_patch(&items[0], "minecraft:stone", 1, None);
        assert_eq!(items[0].get("Slot"), Some(&Tag::Byte(5)));

        assert!(g.run_command("setblock 3 200 3 minecraft:chest"));
        g.world
            .block_entities
            .get_mut(&(3, 200, 3))
            .unwrap()
            .slots_mut()[5] = stack.clone();
        let chunk = Arc::make_mut(g.world.chunks.get_mut(&(0, 0)).unwrap());
        let saved = chunk.saved.get_or_insert_with(Default::default);
        Arc::make_mut(saved).rolled_loot.push(roll.clone());
        let chunks = g.world.chunks.values().cloned().collect::<Vec<_>>();
        let snapshots = g.with_entities(&chunks);
        let encoded = save::encode(&snapshots[0], g.pack.assets.registry, g.res.biomes()).tag;
        println!("H088 unchanged rolled chest full typed={encoded:?}");
        let Tag::List(entities) = encoded.get("block_entities").unwrap() else {
            panic!("block_entities")
        };
        let chest = entities
            .iter()
            .find(|t| t.get("x") == Some(&Tag::Int(3)))
            .unwrap();
        assert_eq!(
            chest.get("LootTable"),
            Some(&Tag::String(roll.table.as_str().into()))
        );
        assert_eq!(chest.get("LootTableSeed"), Some(&Tag::Long(880)));
        assert!(chest.get("Items").is_none());
        let mut view = g.block_entity_nbt([3, 200, 3]).unwrap();
        g.restore_loot_table((3, 200, 3), &mut view);
        println!("H088 unchanged rolled command full typed={view:?}");
        assert_eq!(
            view.get("LootTable"),
            Some(&Tag::String(roll.table.as_str().into()))
        );
        assert!(view.get("Items").is_none());
        assert_eq!(
            g.world.chunks[&(0, 0)].saved.as_ref().unwrap().rolled_loot[0],
            roll
        );
        assert_eq!(
            g.world.block_entities.get(&(3, 200, 3)).unwrap().slots()[5],
            stack
        );
    });
}
