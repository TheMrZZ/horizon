//! #13783: the Speed effect's `minecraft:effect.speed` row lives in the mob's
//! one movement-speed instance. `ClientboundUpdateAttributesPacket` sends each
//! id of that instance once (`AttributeInstance.getModifiers` is the
//! `modifierById` map), a loaded Speed effect has no row (the NBT load runs no
//! `onEffectAdded`), and `/effect clear` removes the row through
//! `onEffectsRemoved`, so the value falls back to the base. Speed II on a pig
//! with base 0.25: `0.25 * (1 + 2 * 0.20000000298023224)`, which `/attribute
//! ... get 1000` reports as 350 in vanilla 26.3.
use super::kill_die_vibration_tests::{PIG, game, run};
use super::lan_attributes::SpeedByType;
use super::*;
use horizon_client::mobs::{
    Species,
    finalize::{self, Attr},
};
use horizon_protocol::packets::play::AttributeSnapshot;

const SPEED_ID: &str = "minecraft:effect.speed";

fn pig_index(game: &Game<'_>) -> usize {
    game.mobs
        .iter()
        .position(|m| m.species() == Species::Pig)
        .expect("the fixture's pig")
}

/// The attribute snapshots the LAN sender builds for the pig.
fn snapshots(game: &Game<'_>) -> Vec<AttributeSnapshot> {
    let m = &game.mobs[pig_index(game)];
    let mut out = Vec::new();
    game.mob_attributes(m, &mut SpeedByType::default(), &mut out);
    out
}

/// The movement-speed snapshot's modifier ids, in sent order.
fn speed_ids(game: &Game<'_>) -> Vec<(String, f64, i32)> {
    let registry = game
        .motion
        .attributes
        .id(Attr::MovementSpeed.id())
        .expect("movement_speed in the catalog")
        .registry_id();
    snapshots(game)
        .into_iter()
        .filter(|s| s.attribute == registry)
        .flat_map(|s| s.modifiers)
        .map(|x| (x.id, x.amount, x.operation))
        .collect()
}

fn speed(game: &Game<'_>) -> f64 {
    finalize::value(&game.mobs[pig_index(game)], Attr::MovementSpeed)
}

const SPEED_II: f64 = 0.25 * (1.0 + 2.0 * 0.20000000298023224);

#[test]
#[ignore = "needs prepared resources in HORIZON_TEST_RESOURCES"]
fn each_movement_speed_id_is_sent_once() {
    let mut game = game();
    assert!(run(&mut game, &format!("effect give {PIG} speed 30 1")).0);
    assert!(
        run(
            &mut game,
            &format!("attribute {PIG} movement_speed modifier add t:manual 0.1 add_value")
        )
        .0
    );
    // A stronger Speed replaces the row in its slot, it adds none. (The same
    // amplifier and duration again would fail: MobEffectInstance.update.)
    assert!(run(&mut game, &format!("effect give {PIG} speed 30 2")).0);
    let ids = speed_ids(&game);
    let mut seen: Vec<&str> = ids.iter().map(|x| x.0.as_str()).collect();
    seen.sort_unstable();
    let count = seen.len();
    seen.dedup();
    assert_eq!(seen.len(), count, "an id sent twice: {ids:?}");
    assert_eq!(
        ids.iter().filter(|x| x.0 == SPEED_ID).collect::<Vec<_>>(),
        [&(SPEED_ID.to_owned(), 3.0 * 0.20000000298023224, 2)]
    );
    assert!(ids.iter().any(|x| x.0 == "t:manual"), "{ids:?}");
    // Every snapshot of the pig holds each id once, not only movement speed.
    for s in snapshots(&game) {
        let mut ids: Vec<_> = s.modifiers.iter().map(|x| x.id.clone()).collect();
        let n = ids.len();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), n, "attribute {}: {:?}", s.attribute, s.modifiers);
    }
}

#[test]
#[ignore = "needs prepared resources in HORIZON_TEST_RESOURCES"]
fn a_loaded_speed_effect_sends_no_row() {
    let mut game = game();
    let i = pig_index(&game);
    // What the `active_effects` load leaves: the effect, and no row.
    game.mobs[i].extra.effects.push(("minecraft:speed", 600, 1));
    assert!(
        speed_ids(&game).iter().all(|x| x.0 != SPEED_ID),
        "{:?}",
        speed_ids(&game)
    );
    assert_eq!(speed(&game), 0.25);
}

#[test]
#[ignore = "needs prepared resources in HORIZON_TEST_RESOURCES"]
fn effect_clear_removes_the_speed_row() {
    let mut game = game();
    assert!(run(&mut game, &format!("effect give {PIG} speed 30 1")).0);
    assert_eq!(speed(&game).to_bits(), SPEED_II.to_bits());
    assert!(
        run(
            &mut game,
            &format!("attribute {PIG} movement_speed modifier value get {SPEED_ID}")
        )
        .0
    );
    // `/effect clear <target> <effect>`.
    assert!(run(&mut game, &format!("effect clear {PIG} speed")).0);
    assert_eq!(speed(&game), 0.25);
    assert!(speed_ids(&game).iter().all(|x| x.0 != SPEED_ID));
    assert!(
        !run(
            &mut game,
            &format!("attribute {PIG} movement_speed modifier value get {SPEED_ID}")
        )
        .0
    );
    // `/effect clear <target>`: every effect, the same removal.
    assert!(run(&mut game, &format!("effect give {PIG} speed 30 1")).0);
    assert_eq!(speed(&game).to_bits(), SPEED_II.to_bits());
    assert!(run(&mut game, &format!("effect clear {PIG}")).0);
    assert_eq!(speed(&game), 0.25);
    assert!(speed_ids(&game).iter().all(|x| x.0 != SPEED_ID));
}
