//! Causal read-only vehicle lookup controls; no additional connected vehicle claim.
use super::{Game, Target};
use crate::game::{Audio, Options, Progress, Settings, load_resources};
use horizon_client::{
    chunk::Chunk,
    lan::{Joined, NetConfig, PlayerSender, guest::Guest, hub::HubConfig},
};
use horizon_core::position::ChunkPos;
use horizon_protocol::packets::configuration::ClientInformation;
use std::sync::{Arc, atomic::AtomicI32, mpsc};

fn with_game(test: impl FnOnce(&mut Game<'_>)) {
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
    options.game_dir =
        Some(std::path::PathBuf::from(std::env::var_os("HORIZON_HOME").unwrap()).join("h007-game"));
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
    game.world_root = game.options.game_dir.as_ref().unwrap().join("world");
    std::fs::create_dir_all(&game.world_root).unwrap();
    game.world
        .chunks
        .insert((0, 0), Arc::new(Chunk::empty(0, 0)));
    game.block_ticks
        .add_empty_chunk(ChunkPos::new(0, 0))
        .unwrap();
    assert!(game.run_command("forceload add 0 0"));
    assert!(game.run_command("fill 0 99 0 15 99 15 stone"));
    test(&mut game);
}

fn add_guest(
    game: &mut Game<'_>,
    conn: u32,
    name: &str,
    entity: i32,
) -> mpsc::Receiver<horizon_client::lan::Out> {
    let (tx, rx) = mpsc::channel();
    let uuid = horizon_protocol::crypto::offline_uuid(name);
    let mut guest = Guest::new(
        Joined {
            conn,
            uuid,
            name: name.into(),
            properties: Vec::new(),
            info: ClientInformation::new(2),
            brand: "test".into(),
            sender: PlayerSender::new(tx, Arc::default()),
            latency: Arc::new(AtomicI32::new(0)),
            address: "127.0.0.1:25565".parse().unwrap(),
            expected: false,
            configured: Default::default(),
        },
        entity,
        [4.5 + f64::from(conn), 100.0, 4.5],
        2,
    );
    guest.dimension = "minecraft:overworld".into();
    guest.placed = true;
    guest.loaded = true;
    guest.game_mode = 0;
    let host = game.lan.as_mut().unwrap();
    host.hub.guests.push(guest);
    host.hub.ops.insert(uuid, 4);
    host.hub.refresh_permissions();
    assert_eq!(host.hub.guest(conn).unwrap().permission, 4);
    rx
}

#[test]
#[ignore = "needs prepared resources; causal lookup inputs through actual registered Guest contexts"]
fn guest_vehicle_lookup_preserves_current_context_and_missing_records() {
    with_game(|game| {
        game.lan_host(
            crate::game::lan::LanSettings {
                port: 0,
                guest_commands: true,
                force_game_mode: false,
            },
            NetConfig {
                announce: false,
                ..NetConfig::loopback(0, "h007".into(), 2)
            },
            HubConfig {
                default_game_mode: 0,
                guest_commands: true,
                force_game_mode: false,
                hardcore: false,
                view_distance: 2,
                simulation_distance: 2,
                dimension: "minecraft:overworld".into(),
                allow_flight: false,
                seed: 0,
                max_players: 3,
                integrated: false,
                levels: Default::default(),
            },
        )
        .unwrap();
        let _receivers = [
            add_guest(game, 2, "Alice", 31),
            add_guest(game, 3, "Bob", 32),
            add_guest(game, 4, "Carol", 33),
        ];
        let source = game.server_source();
        assert!(game.player.is_none() && source.entity.is_none());
        let clock = game.world.game_time;
        for (n, x, z) in [
            (11, 8.5, 8.5),
            (12, 10.5, 8.5),
            (13, 12.5, 8.5),
            (14, 8.5, 10.5),
            (15, 10.5, 10.5),
        ] {
            assert!(game.run_command(&format!(
                "summon minecraft:pig {x} 100 {z} {{UUID:[I;73,7,9,{n}],NoAI:1b,NoGravity:1b}}"
            )));
        }
        let ids = [11, 12, 13, 14, 15].map(|n| {
            game.mobs
                .iter()
                .find(|m| game.persist.uuid_of(m.id) == Some([73, 7, 9, n]))
                .unwrap()
                .id
        });
        let [
            alice_vehicle,
            bob_vehicle,
            new_vehicle,
            mob_rider,
            mob_vehicle,
        ] = ids;
        let uuid = |n| format!("00000049-0000-0007-0000-0009000000{n:02x}");
        assert_eq!(
            game.execute_line(&format!("ride {} mount {}", uuid(14), uuid(15)), source)
                .unwrap(),
            1
        );
        assert_eq!(
            game.ride_vehicle(Target::Mob(mob_rider)),
            Some(Target::Mob(mob_vehicle))
        );
        assert_eq!(game.ride_vehicle(Target::Mob(new_vehicle)), None);
        assert_eq!(game.ride_vehicle(Target::Mob(u32::MAX)), None);
        assert_eq!(
            game.execute_line(&format!("ride Alice mount {}", uuid(11)), source)
                .unwrap(),
            1
        );
        assert_eq!(
            game.execute_line(&format!("ride Bob mount {}", uuid(12)), source)
                .unwrap(),
            1
        );
        assert_eq!(
            game.ride_vehicle(Target::Guest(2)),
            Some(Target::Mob(alice_vehicle))
        );
        assert_eq!(
            game.ride_vehicle(Target::Guest(3)),
            Some(Target::Mob(bob_vehicle))
        );
        assert_eq!(game.ride_vehicle(Target::Guest(4)), None);
        assert_eq!(game.ride_vehicle(Target::Guest(99)), None);
        assert_eq!(game.ride_vehicle(Target::Player), None);
        game.for_player(Target::Guest(2), |g| {
            assert_eq!(g.riding, Some(alice_vehicle));
            assert_eq!(g.command_guest(2).unwrap().vehicle, Some(alice_vehicle));
            g.stop_riding();
            // The saved record is deliberately stale until leave_guest commits this turn.
            assert_eq!(g.riding, None);
            assert_eq!(g.command_guest(2).unwrap().vehicle, Some(alice_vehicle));
            assert_eq!(g.ride_vehicle(Target::Guest(2)), None);
            assert_eq!(g.ride_vehicle(Target::Player), None);
            g.ride_mount(Target::Player, Target::Mob(new_vehicle))
                .unwrap();
            assert_eq!(g.riding, Some(new_vehicle));
            assert_eq!(g.command_guest(2).unwrap().vehicle, Some(alice_vehicle));
            let before = (
                g.riding,
                g.command_guest(2).unwrap().vehicle,
                g.command_guest(3).unwrap().vehicle,
            );
            assert_eq!(
                g.ride_vehicle(Target::Guest(2)),
                Some(Target::Mob(new_vehicle))
            );
            assert_eq!(
                g.ride_vehicle(Target::Player),
                Some(Target::Mob(new_vehicle))
            );
            assert_eq!(
                g.ride_vehicle(Target::Guest(3)),
                Some(Target::Mob(bob_vehicle))
            );
            assert_eq!(g.ride_vehicle(Target::Guest(4)), None);
            assert_eq!(g.ride_vehicle(Target::Guest(99)), None);
            assert_eq!(
                g.ride_vehicle(Target::Mob(mob_rider)),
                Some(Target::Mob(mob_vehicle))
            );
            assert_eq!(g.ride_vehicle(Target::Mob(u32::MAX)), None);
            assert_eq!(
                (
                    g.riding,
                    g.command_guest(2).unwrap().vehicle,
                    g.command_guest(3).unwrap().vehicle
                ),
                before,
                "lookup must not swap or write state"
            );
        })
        .unwrap();
        assert_eq!(game.command_guest(2).unwrap().vehicle, Some(new_vehicle));
        assert_eq!(game.command_guest(3).unwrap().vehicle, Some(bob_vehicle));
        assert_eq!(game.command_guest(4).unwrap().vehicle, None);
        assert!(game.player.is_none() && game.riding.is_none() && game.source_entity().is_none());
        assert_eq!(game.world.game_time, clock);
    });
}
