//! Postbaseline owner-identity correction controls. Not a Guest or native-hook parity claim.
use super::*;
use crate::game::native_machines::tests::server_game;
use horizon_client::items::ItemStack;
use horizon_client::lan::{Joined, PlayerSender, guest::Guest};
use horizon_core::position::BlockPos;
use horizon_protocol::packets::configuration::ClientInformation;

#[test]
#[ignore = "needs prepared resources and scratch homes/cwd; added after FIRST FIX FAIL"]
fn actual_owner_guest_turn_uses_exact_binding_and_restores_recipient() {
    let mut game = server_game();
    game.world_root = std::env::current_dir()
        .unwrap()
        .join("owner-publication-world");
    let pack = game.world_root.join("datapacks/owner_publication");
    let dir = pack.join("data/owner_publication/advancement");
    std::fs::create_dir_all(&dir).unwrap();
    let f = horizon_core::version::DATA_PACK_FORMAT;
    std::fs::write(
        pack.join("pack.mcmeta"),
        serde_json::json!({"pack":{
        "description":"Actual owner-turn publication controls","pack_format":f.0,
        "min_format":[f.0,f.1],"max_format":[f.0,f.1]}})
        .to_string(),
    )
    .unwrap();
    std::fs::write(
        dir.join("single.json"),
        serde_json::json!({"criteria":{
        "seen":{"trigger":"minecraft:inventory_changed","conditions":{
            "items":[{"items":"minecraft:diamond"}]}}}})
        .to_string(),
    )
    .unwrap();
    game.build_registries();
    game.set_allow_commands(true);
    assert!(game.run_command("reload"));
    assert!(game.adv.tree.get("owner_publication:single").is_some());
    game.block_ticks
        .add_empty_chunk(BlockPos::new(0, 0, 0).chunk())
        .unwrap();
    assert!(game.run_command("fill 0 199 0 15 199 15 stone"));
    game.lan_host(
        crate::game::lan::LanSettings {
            port: 0,
            guest_commands: true,
            force_game_mode: false,
        },
        NetConfig {
            announce: false,
            ..NetConfig::loopback(0, "owner-publication".into(), 2)
        },
        HubConfig {
            default_game_mode: 1,
            guest_commands: true,
            force_game_mode: false,
            hardcore: false,
            view_distance: 2,
            simulation_distance: 2,
            dimension: "minecraft:overworld".into(),
            allow_flight: false,
            seed: 0,
            max_players: 8,
            integrated: true,
            levels: Default::default(),
        },
    )
    .unwrap();
    game.inventory[0] = Some(ItemStack::new("minecraft:stick", 7));
    let host_uuid = game.level.player_uuid;
    let clock = game.world.game_time;
    assert!(game.world.loaded(4, 4));
    assert_eq!(
        &*game.res.facts().get(game.world.block(4, 199, 4)).name,
        "minecraft:stone"
    );
    let mut received = Vec::new();
    for (index, owner, binding, uuid, owner_exception) in [
        (0, Some(111), true, 111, true),
        (1, Some(111), true, 222, false),
        (2, Some(111), false, 111, false),
        (3, None, true, 111, false),
        (4, None, false, 111, false),
    ] {
        game.serving = owner.map(|owner| {
            let (_tx, tasks) = channel();
            Serving {
                flags: Arc::new(Flags::default()),
                tasks,
                owner,
                pending: None,
                load: LoadPhase::default(),
            }
        });
        let conn = index + 2;
        let (tx, rx) = channel();
        received.push(rx);
        let mut guest = Guest::new(
            Joined {
                conn,
                uuid,
                name: "SameShape".into(),
                properties: Vec::new(),
                info: ClientInformation::new(2),
                brand: "test".into(),
                sender: PlayerSender::new(tx, Arc::default()),
                latency: Arc::new(AtomicI32::new(0)),
                address: "127.0.0.1:25565".parse().unwrap(),
                expected: false,
                configured: Default::default(),
            },
            conn as i32,
            [4.5, 200.0, 4.5],
            2,
        );
        guest.dimension = "minecraft:overworld".into();
        guest.placed = true;
        guest.loaded = true;
        guest.permission = 2;
        let mut host = game.lan.take().unwrap();
        host.hub.guests.push(guest);
        assert!(game.lan_acting.is_none() && game.lan_acting_uuid.is_none());
        assert!(game.command_source.is_none());
        let swap = game
            .enter_guest(&mut host, conn, game.dim)
            .expect("actual Guest turn");
        game.lan = Some(host);
        assert_eq!(game.lan_acting, Some(conn));
        assert_eq!(game.lan_acting_uuid, Some(uuid));
        assert_eq!(game.acting_player_name(), "SameShape");
        let actual_binding = game.lan_acting_uuid;
        if !binding {
            game.lan_acting_uuid = None;
        }
        assert!(game.run_command("item replace entity @s inventory.0 with diamond 1"));
        assert!(game.run_command(
            "execute if entity @s[advancements={owner_publication:single={seen=true}}]"
        ));
        assert!(game.run_command("advancement revoke @s only owner_publication:single seen"));
        assert!(game.run_command(
            "execute unless entity @s[advancements={owner_publication:single={seen=true}}]"
        ));
        assert!(game.run_command("item replace entity @s inventory.1 with stone 1"));
        let tree = game.adv.tree.clone();
        let criterion = tree.get("owner_publication:single").unwrap();
        let granted = game.adv.progress.done(criterion);
        assert_eq!(
            granted, !owner_exception,
            "only bound owner reaches narrow changed-stack dispatch; excluded cases retain legacy regrant"
        );
        if !binding || owner.is_none() {
            if granted {
                assert!(
                    game.run_command("advancement revoke @s only owner_publication:single seen")
                );
            }
            assert!(game.run_command("clear @s"));
            assert!(game.run_command("item replace entity @s inventory.1 with stone 1"));
            assert!(
                !game.adv.progress.done(criterion),
                "absent identity must not manufacture a matching item or grant"
            );
        }
        assert!(game.command_source.is_none());
        game.lan_acting_uuid = actual_binding;
        let progress = game.adv.progress.clone();
        let mut host = game.lan.take().unwrap();
        game.leave_guest(&mut host, swap);
        assert!(
            game.lan_acting.is_none()
                && game.lan_acting_uuid.is_none()
                && game.lan_acting_name.is_none()
        );
        assert!(game.command_source.is_none());
        assert!(game.player.is_none());
        assert_eq!(game.level.player_uuid, host_uuid);
        let sentinel = game.inventory[0].as_ref().unwrap();
        assert_eq!((&*sentinel.item, sentinel.count), ("minecraft:stick", 7));
        assert_eq!(
            host.hub.guest(conn).unwrap().advancements.done(criterion),
            progress.done(criterion)
        );
        assert!(
            !game.adv.progress.done(criterion),
            "host never receives the Guest's criterion"
        );
        for previous in 2..conn {
            // Previously restored recipients keep their own independent inventory.
            let saved = host.hub.guest(previous).unwrap();
            assert_eq!(saved.uuid, if previous == 3 { 222 } else { 111 });
        }
        game.lan = Some(host);
    }
    assert_eq!(game.world.game_time, clock);
}
