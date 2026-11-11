use super::*;
use crate::game::{Audio, Options, Progress, Settings, load_resources};
use horizon_client::motion::PlayerMotion;
pub(in crate::game) fn with_game(test: impl FnOnce(&mut Game<'_>)) {
    let root = std::path::PathBuf::from(std::env::var_os("HORIZON_TEST_RESOURCES").unwrap());
    let settings = Settings::default();
    let loaded = load_resources(
        root.clone(),
        None,
        &root,
        &settings,
        std::time::Instant::now(),
        false,
        true,
        &Progress::new(),
        &mut |_, _| unreachable!(),
    )
    .unwrap();
    let mut options = Options::parse(std::iter::empty()).unwrap().unwrap();
    options.resources = root.clone();
    options.game_dir = Some(
        std::path::PathBuf::from(std::env::var_os("HORIZON_HOME").unwrap())
            .join(format!("h161-{:?}", std::thread::current().id())),
    );
    options.execution_role = crate::game::execution_role::ExecutionRole::IntegratedServer;
    let mut game = Game::new(
        &loaded.resources,
        loaded.pack,
        loaded.gui.unwrap(),
        options,
        settings,
        Audio::silent(&root),
        None,
    )
    .unwrap();
    game.build_registries();
    game.set_allow_commands(true);
    game.world
        .chunks
        .insert((0, 0), Arc::new(Chunk::empty(0, 0)));
    game.block_ticks
        .add_empty_chunk(ChunkPos::new(0, 0))
        .unwrap();
    assert!(game.run_command("forceload add 0 0"));
    game.player = Some(PlayerMotion::new(game.motion, Vec3::new(2.5, 200.0, 2.5)).unwrap());
    game.creative = true;
    let time = game.world.game_time;
    test(&mut game);
    assert_eq!(game.world.game_time, time);
}

#[test]
#[ignore = "prepared resources; H088 actual Player save projection baseline"]
fn prototype_equal_stack_patch_is_elided_from_actual_player_save() {
    with_game(|g| {
        assert!(g.run_command("item replace entity @s hotbar.0 with minecraft:stone[minecraft:max_stack_size=64,minecraft:custom_data={keep:881}] 2"));
        let before = g.inventory[0].as_ref().unwrap().clone();
        assert_eq!(before.count, 2);
        g.sync_level();
        let saved = g.level.player_tag();
        println!(
            "physical live={before:?} saved={saved:?} clock={}",
            g.world.game_time
        );
        let Tag::List(items) = saved.get("Inventory").unwrap() else {
            panic!("actual Inventory list")
        };
        let stack = items
            .iter()
            .find(|t| t.get("Slot") == Some(&Tag::Byte(0)))
            .unwrap();
        assert_eq!(
            stack.get("id"),
            Some(&Tag::String("minecraft:stone".into()))
        );
        assert_eq!(stack.get("count"), Some(&Tag::Int(2)));
        let components = stack.get("components").unwrap();
        assert!(components.get("minecraft:custom_data").is_some());
        assert!(
            components.get("minecraft:max_stack_size").is_none(),
            "actual Player saved patch must elide prototype64; live stack remains unchanged"
        );
        assert_eq!(g.inventory[0].as_ref().unwrap(), &before);
    });
}

fn assert_patch(stack: &Tag, id: &str, count: i32, size: Option<i32>) {
    assert_eq!(stack.get("id"), Some(&Tag::String(id.into())));
    assert_eq!(stack.get("count"), Some(&Tag::Int(count)));
    let actual = stack
        .get("components")
        .and_then(|c| c.get("minecraft:max_stack_size"));
    assert_eq!(
        actual,
        size.map(Tag::Int).as_ref(),
        "full typed root: {stack:?}"
    );
}

#[test]
#[ignore = "prepared resources; H088 actual loaded saved-stack families"]
fn full_saved_stack_roots_keep_live_state_and_nested_templates() {
    use crate::game::commands::Target;
    use horizon_client::save;
    with_game(|g| {
        for (slot, id, size) in [
            (0, "minecraft:stone", 64),
            (1, "minecraft:ender_pearl", 16),
            (2, "minecraft:iron_helmet", 1),
        ] {
            assert_eq!(g.items().get(id).unwrap().max_stack, size);
            let count = size.min(2);
            let command = format!(
                "item replace entity @s hotbar.{slot} with {id}[minecraft:max_stack_size={size},minecraft:custom_data={{keep:882}}] {count}"
            );
            println!("H088 setup {command}");
            assert!(g.run_command(&command));
        }
        assert!(g.run_command(
            "item replace entity @s hotbar.3 with minecraft:stone[minecraft:max_stack_size=7] 2"
        ));
        assert!(g.run_command(
            "item replace entity @s hotbar.4 with minecraft:stone[!minecraft:max_stack_size] 2"
        ));
        assert!(g.run_command("item replace entity @s hotbar.5 with minecraft:shulker_box[minecraft:container=[{slot:0,item:{id:'minecraft:stone',count:1,components:{'minecraft:max_stack_size':64}}}]]"));
        let live = g.inventory.clone();
        g.armor[0] = g.inventory[2].clone();
        g.offhand = g.inventory[1].clone();
        g.ender_items[5] = g.inventory[0].clone();
        g.sync_level();
        let player = g.level.player_tag();
        println!("H088 Player typed={player:?} clock={}", g.world.game_time);
        let Tag::List(inv) = player.get("Inventory").unwrap() else {
            panic!("Inventory")
        };
        for (slot, id) in [
            (0, "minecraft:stone"),
            (1, "minecraft:ender_pearl"),
            (2, "minecraft:iron_helmet"),
        ] {
            let stack = inv
                .iter()
                .find(|s| s.get("Slot") == Some(&Tag::Byte(slot)))
                .unwrap();
            assert_patch(stack, id, if slot == 2 { 1 } else { 2 }, None);
            assert!(
                stack
                    .get("components")
                    .unwrap()
                    .get("minecraft:custom_data")
                    .is_some()
            );
        }
        assert_patch(
            inv.iter()
                .find(|s| s.get("Slot") == Some(&Tag::Byte(3)))
                .unwrap(),
            "minecraft:stone",
            2,
            Some(7),
        );
        assert!(
            inv.iter()
                .find(|s| s.get("Slot") == Some(&Tag::Byte(4)))
                .unwrap()
                .get("components")
                .unwrap()
                .get("!minecraft:max_stack_size")
                .is_some()
        );
        let raw = save::item_to_nbt(live[5].as_ref().unwrap());
        assert_eq!(
            inv.iter()
                .find(|s| s.get("Slot") == Some(&Tag::Byte(5)))
                .unwrap()
                .get("components"),
            raw.get("components"),
            "raw nested template stays raw"
        );
        assert_patch(
            player.get("equipment").unwrap().get("head").unwrap(),
            "minecraft:iron_helmet",
            1,
            None,
        );
        assert_patch(
            player.get("equipment").unwrap().get("offhand").unwrap(),
            "minecraft:ender_pearl",
            2,
            None,
        );
        let Tag::List(ender) = player.get("EnderItems").unwrap() else {
            panic!("EnderItems")
        };
        assert_patch(&ender[0], "minecraft:stone", 2, None);
        assert_eq!(g.inventory, live, "live source is not normalized by save");

        assert!(g.run_command("setblock 3 200 3 minecraft:chest"));
        assert!(g.run_command("item replace block 3 200 3 container.5 with minecraft:stone[minecraft:max_stack_size=64,minecraft:custom_data={keep:883}] 2"));
        let before = g.world.block_entities.get(&(3, 200, 3)).unwrap().slots()[5].clone();
        let chunks: Vec<_> = g.world.chunks.values().cloned().collect();
        let snapshots = g.with_entities(&chunks);
        let snapshot = snapshots
            .iter()
            .find(|c| c.block_entities.iter().any(|(p, _)| *p == (3, 200, 3)))
            .unwrap();
        let encoded = save::encode(snapshot, g.pack.assets.registry, g.res.biomes()).tag;
        println!("H088 chunk typed={encoded:?}");
        let Tag::List(entities) = encoded.get("block_entities").unwrap() else {
            panic!("block_entities")
        };
        let chest = entities
            .iter()
            .find(|e| e.get("x") == Some(&Tag::Int(3)))
            .unwrap();
        let Tag::List(stacks) = chest.get("Items").unwrap() else {
            panic!("Items")
        };
        assert_patch(&stacks[0], "minecraft:stone", 2, None);
        let (_, reloaded) = save::decode_block_entity(chest).unwrap();
        assert_patch(
            &save::item_to_nbt(reloaded.slots()[5].as_ref().unwrap()),
            "minecraft:stone",
            2,
            None,
        );
        assert_eq!(
            g.world.block_entities.get(&(3, 200, 3)).unwrap().slots()[5],
            before
        );
        let template = crate::game::structure_blocks::tests::captured_box(g);
        println!("H088 captured template typed={template:?}");
        let Tag::List(blocks) = template.get("blocks").unwrap() else {
            panic!("template blocks")
        };
        let Tag::List(stacks) = blocks[0].get("nbt").unwrap().get("Items").unwrap() else {
            panic!("template Items")
        };
        assert_patch(&stacks[0], "minecraft:stone", 2, None);
        assert!(g.run_command("setblock 4 200 3 minecraft:lectern"));
        assert!(g.run_command("data merge block 4 200 3 {Book:{id:'minecraft:writable_book',count:1,components:{'minecraft:max_stack_size':1,'minecraft:custom_data':{keep:890}}}}"));
        let book = g.block_entity_nbt([4, 200, 3]).unwrap();
        assert_patch(
            book.get("Book").unwrap(),
            "minecraft:writable_book",
            1,
            None,
        );
        assert!(g.run_command("setblock 7 200 3 minecraft:decorated_pot"));
        assert!(g.run_command("data merge block 7 200 3 {item:{id:'minecraft:stone',count:2,components:{'minecraft:max_stack_size':64,'minecraft:custom_data':{keep:891}}},sherds:{back:{id:'minecraft:brick',components:{'minecraft:max_stack_size':64}}}}"));
        let pot = g.pot_data([7, 200, 3]).unwrap();
        println!("H088 pot typed={pot:?}");
        assert_patch(pot.get("item").unwrap(), "minecraft:stone", 2, None);
        assert_eq!(
            pot.get("sherds")
                .unwrap()
                .get("back")
                .unwrap()
                .get("components")
                .unwrap()
                .get("minecraft:max_stack_size"),
            Some(&Tag::Int(64)),
            "pot sherd is a raw template"
        );
        let raw = crate::game::spill::saved_entity(&g.world, [7, 200, 3]).unwrap();
        assert_eq!(
            raw.get("item")
                .unwrap()
                .get("components")
                .unwrap()
                .get("minecraft:max_stack_size"),
            Some(&Tag::Int(64)),
            "read-only save view retains the live raw store"
        );
        let snapshots = g.with_entities(&g.world.chunks.values().cloned().collect::<Vec<_>>());
        let encoded = save::encode(&snapshots[0], g.pack.assets.registry, g.res.biomes()).tag;
        let Tag::List(entities) = encoded.get("block_entities").unwrap() else {
            panic!("block entities")
        };
        let pot = entities
            .iter()
            .find(|p| p.get("x") == Some(&Tag::Int(7)))
            .unwrap();
        assert_patch(pot.get("item").unwrap(), "minecraft:stone", 2, None);
        assert!(g.run_command("summon armor_stand 5 200 5 {NoGravity:1b}"));
        assert!(g.run_command("item replace entity @e[type=minecraft:armor_stand,limit=1] weapon.mainhand with minecraft:ender_pearl[minecraft:max_stack_size=16,minecraft:custom_data={keep:884}] 2"));
        let id = g.armor_stands.last().unwrap().id;
        let stand = g.entity_nbt(Target::Stand(id)).unwrap();
        println!("H088 stand typed={stand:?}");
        assert_patch(
            stand.get("equipment").unwrap().get("mainhand").unwrap(),
            "minecraft:ender_pearl",
            2,
            None,
        );
        assert!(g.run_command("summon horse 6 200 6 {NoAI:1b,NoGravity:1b,equipment:{body:{id:'minecraft:diamond_horse_armor',count:1,components:{'minecraft:max_stack_size':7,'minecraft:custom_data':{keep:885}}}}}"));
        let id = g.mobs.last().unwrap().id;
        let horse = g.entity_nbt(Target::Mob(id)).unwrap();
        println!("H088 horse typed={horse:?}");
        assert_patch(
            horse.get("equipment").unwrap().get("body").unwrap(),
            "minecraft:diamond_horse_armor",
            1,
            Some(7),
        );
        assert!(
            horse
                .get("equipment")
                .unwrap()
                .get("body")
                .unwrap()
                .get("components")
                .unwrap()
                .get("minecraft:custom_data")
                .is_some()
        );
    });
}

#[test]
#[ignore = "prepared resources; H088 new Hotbar capture versus retained raw rows"]
fn hotbar_capture_normalizes_only_the_new_actual_row() {
    use horizon_client::{hotbar, save};
    with_game(|g| {
        assert!(g.run_command("item replace entity @s hotbar.0 with minecraft:stone[minecraft:max_stack_size=64,minecraft:custom_data={keep:886}] 2"));
        let original = g.inventory[0].clone();
        let path = Game::options_path().with_file_name("hotbar.nbt");
        let mut rows = vec![vec![None; hotbar::SIZE]; hotbar::GROUPS];
        rows[1][0] = original.clone();
        hotbar::save(&path, &rows).unwrap();
        g.hotbar_load_or_save(0, false);
        let file = save::read_dat(&path).unwrap();
        println!("H088 Hotbar typed={file:?}");
        let Tag::List(new) = file.get("0").unwrap() else {
            panic!("new row")
        };
        assert_patch(&new[0], "minecraft:stone", 2, None);
        let Tag::List(old) = file.get("1").unwrap() else {
            panic!("old row")
        };
        assert_patch(&old[0], "minecraft:stone", 2, Some(64));
        assert_eq!(g.inventory[0], original);
        g.hotbar_load_or_save(0, true);
        assert_eq!(g.inventory[0].as_ref().unwrap().count, 2);
    });
}

#[path = "max_stack_saved_tests/saved_roots_tests.rs"]
mod saved_roots_tests;

#[path = "max_stack_saved_tests/rolled_loot_tests.rs"]
mod rolled_loot_tests;
