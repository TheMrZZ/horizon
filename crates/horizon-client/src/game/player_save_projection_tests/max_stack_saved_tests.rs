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
