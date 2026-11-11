//! #13783: the movement-speed instance as one keyed store (`AttributeInstance`
//! of the pinned 26.3 server: an active map with one row per id, transient rows
//! that `pack` does not save, a permanent row that `addOrUpdateTransientModifier`
//! leaves in `permanentModifiers`), the state transitions that add and remove
//! the vanilla rows, and the Speed effect's stored row. The amounts are the
//! double literals of the 26.3 bytecode; the Java replies 350 and 400 are the
//! measured `/attribute` answers for a NoAI pig with Speed II.
use super::*;
use crate::mobs::{Mob, Who};
use std::{collections::HashMap, sync::Arc};

fn mob(species: Species, id: &str, speed: f64) -> Mob {
    let mut k = (*crate::mobs::sensor_scan_tests::kind()).clone();
    k.id = id.into();
    k.species = species;
    k.attributes = Arc::new(HashMap::from([(
        "minecraft:movement_speed".to_string(),
        speed,
    )]));
    Mob::new(1, Arc::new(k), horizon_core::geometry::Vec3::ZERO, 0.0)
}
fn rows(m: &Mob) -> Vec<(String, f64, bool)> {
    m.extra.attributes.as_ref().map_or(Vec::new(), |s| {
        s.active()
            .filter(|x| x.attr == Attr::MovementSpeed)
            .map(|x| (x.id, x.amount, x.transient))
            .collect()
    })
}
fn saved(m: &Mob) -> Vec<(String, f64)> {
    m.extra.attributes.as_ref().map_or(Vec::new(), |s| {
        saved_modifiers(s)
            .filter(|x| x.attr == Attr::MovementSpeed)
            .map(|x| (x.id, x.amount))
            .collect()
    })
}
/// The ids of one operation's bucket in iteration order (calculation view).
fn bucket(m: &Mob, op: Operation) -> Vec<(String, f64)> {
    m.extra
        .attributes
        .as_ref()
        .and_then(|s| s.instance(Attr::MovementSpeed))
        .map_or(Vec::new(), |i| {
            i.operation_modifiers(op)
                .map(|x| (x.id().to_owned(), x.amount()))
                .collect()
        })
}

#[test]
fn the_store_holds_one_active_row_per_id() {
    let mut m = mob(Species::Pig, "minecraft:pig", 0.25);
    add_modifier(&mut m, Attr::MovementSpeed, "t:x", 0.1, Op::AddValue);
    add_modifier(&mut m, Attr::MovementSpeed, "t:x", 0.2, Op::AddValue);
    assert_eq!(rows(&m), [("t:x".to_string(), 0.2, false)]);
    remove_modifier(&mut m, Attr::MovementSpeed, "t:x");
    add_transient(&mut m, Attr::MovementSpeed, "t:x", 0.3, Op::AddValue);
    assert_eq!(rows(&m), [("t:x".to_string(), 0.3, true)]);
    // A transient row is not saved.
    assert!(saved(&m).is_empty());
}

/// `addOrUpdateTransientModifier` puts the id into the active map and the NEW
/// operation's bucket but never removes the old bucket's entry, and leaves
/// `permanentModifiers` alone (26.3 AttributeInstance 158-183): a permanent
/// ADD_VALUE row overlaid by a transient ADD_MULTIPLIED_BASE one with the same
/// id counts in both buckets, while the active view and the save each hold
/// one row.
#[test]
fn an_overlaid_permanent_row_keeps_its_old_operation_bucket() {
    const ID: &str = "minecraft:suffocating";
    let base = 0.175;
    let mut m = mob(Species::Strider, "minecraft:strider", base);
    add_modifier(&mut m, Attr::MovementSpeed, ID, 0.1, Op::AddValue);
    let amount = f64::from(-0.34f32);
    add_or_update_transient(
        &mut m,
        Attr::MovementSpeed,
        ID,
        amount,
        Op::AddMultipliedBase,
    );
    add_or_update_transient(
        &mut m,
        Attr::MovementSpeed,
        ID,
        amount,
        Op::AddMultipliedBase,
    );
    // Active (LAN, /attribute) and saved views: one row each.
    assert_eq!(rows(&m), [(ID.to_string(), amount, true)]);
    assert_eq!(saved(&m), [(ID.to_string(), 0.1)]);
    // Calculation buckets: the old one keeps 0.1, the new one holds -0.34F once.
    assert_eq!(bucket(&m, Operation::AddValue), [(ID.to_string(), 0.1)]);
    assert_eq!(
        bucket(&m, Operation::AddMultipliedBase),
        [(ID.to_string(), amount)]
    );
    let added = base + 0.1;
    assert_eq!(value(&m, Attr::MovementSpeed), added + added * amount);
    // removeModifier(id) removes the active row, its own bucket's entry and the
    // permanent one; the old ADD_VALUE bucket entry stays, as in Java.
    remove_modifier(&mut m, Attr::MovementSpeed, ID);
    assert!(rows(&m).is_empty() && saved(&m).is_empty());
    assert_eq!(bucket(&m, Operation::AddValue), [(ID.to_string(), 0.1)]);
    assert!(bucket(&m, Operation::AddMultipliedBase).is_empty());
    assert_eq!(value(&m, Attr::MovementSpeed), base + 0.1);
    // addOrReplacePermanentModifier: remove(id), then add: the ADD_VALUE put
    // replaces the stale entry with the same id.
    add_modifier(&mut m, Attr::MovementSpeed, ID, 0.2, Op::AddValue);
    assert_eq!(rows(&m), [(ID.to_string(), 0.2, false)]);
    assert_eq!(saved(&m), [(ID.to_string(), 0.2)]);
    assert_eq!(bucket(&m, Operation::AddValue), [(ID.to_string(), 0.2)]);
}

/// A save and load of the overlaid strider (`pack`, then `apply` on a fresh
/// mob): only the permanent ADD_VALUE row comes back.
#[test]
fn the_save_of_an_overlaid_row_is_the_permanent_one() {
    const ID: &str = "minecraft:suffocating";
    let mut m = mob(Species::Strider, "minecraft:strider", 0.175);
    add_modifier(&mut m, Attr::MovementSpeed, ID, 0.1, Op::AddValue);
    crate::mobs::strider::set_suffocating(&mut m, true);
    let packed: Vec<(String, f64, Op)> = m
        .extra
        .attributes
        .as_ref()
        .unwrap()
        .saved()
        .map(|x| (x.id, x.amount, x.op))
        .collect();
    let mut loaded = mob(Species::Strider, "minecraft:strider", 0.175);
    apply_saved(&mut loaded, Attr::MovementSpeed, &packed);
    assert_eq!(rows(&loaded), [(ID.to_string(), 0.1, false)]);
    assert_eq!(
        bucket(&loaded, Operation::AddValue),
        [(ID.to_string(), 0.1)]
    );
    assert!(bucket(&loaded, Operation::AddMultipliedBase).is_empty());
}

/// Two legal manual ids in one fastutil slot (`t:a` and `t:a2` both hash to
/// slot 3 of the 32): the later one probes to slot 4 and is iterated first;
/// removing `t:a` shifts `t:a2` back into slot 3 (`shiftKeys`), so `t:a` added
/// again lands in slot 4 and is now first. The history decides the order.
#[test]
fn colliding_manual_ids_follow_the_maps_removal_history() {
    let mut m = mob(Species::Pig, "minecraft:pig", 0.25);
    add_modifier(&mut m, Attr::MovementSpeed, "t:a", 0.1, Op::AddValue);
    add_modifier(&mut m, Attr::MovementSpeed, "t:a2", 0.2, Op::AddValue);
    let ids = |m: &Mob| -> Vec<String> {
        bucket(m, Operation::AddValue)
            .into_iter()
            .map(|x| x.0)
            .collect()
    };
    assert_eq!(ids(&m), ["t:a2", "t:a"]);
    remove_modifier(&mut m, Attr::MovementSpeed, "t:a");
    add_modifier(&mut m, Attr::MovementSpeed, "t:a", 0.1, Op::AddValue);
    assert_eq!(ids(&m), ["t:a", "t:a2"]);
}

#[test]
fn a_given_speed_effect_stores_its_row_and_a_loaded_one_none() {
    let mut m = mob(Species::Pig, "minecraft:pig", 0.25);
    m.add_effect("minecraft:speed", -1, 1);
    let row = modifier(&m, Attr::MovementSpeed, EFFECT_SPEED_ID).expect("the Speed row");
    // MobEffects: 0.20000000298023224d per level; Speed II is 0.4000000059604645.
    assert_eq!(row.amount.to_bits(), 0x3FD9_9999_A000_0000);
    assert_eq!((row.op, row.transient), (Op::AddMultipliedTotal, false));
    // Java: modifier value get ... 1000 is 400, get 1000 is 350.
    assert_eq!((row.amount * 1000.0) as i32, 400);
    assert_eq!((m.movement_speed() * 1000.0) as i32, 350);
    assert_eq!(m.movement_speed(), value(&m, Attr::MovementSpeed));
    // A stronger one replaces the row: one row, three levels.
    m.add_effect("minecraft:speed", -1, 2);
    assert_eq!(rows(&m).len(), 1);
    assert_eq!(
        modifier_amount(&m, Attr::MovementSpeed, EFFECT_SPEED_ID),
        Some(f64::from(0.2f32) * 3.0)
    );
    // Its end removes it (onEffectsRemoved).
    effect_attributes_changed(&mut m, "minecraft:speed", None);
    assert!(!has_modifier(&m, Attr::MovementSpeed, EFFECT_SPEED_ID));
    // A loaded active_effects entry adds nothing: Java get 1000 is 250.
    let mut loaded = mob(Species::Pig, "minecraft:pig", 0.25);
    loaded.extra.effects.push(("minecraft:speed", -1, 1));
    assert!(!has_modifier(&loaded, Attr::MovementSpeed, EFFECT_SPEED_ID));
    assert_eq!((loaded.movement_speed() * 1000.0) as i32, 250);
}

#[test]
fn a_manual_effect_speed_row_shares_the_effects_single_slot() {
    let mut m = mob(Species::Pig, "minecraft:pig", 0.25);
    add_modifier(
        &mut m,
        Attr::MovementSpeed,
        EFFECT_SPEED_ID,
        0.1,
        Op::AddValue,
    );
    m.add_effect("minecraft:speed", -1, 1);
    assert_eq!(rows(&m).len(), 1);
    assert_eq!(
        modifier_amount(&m, Attr::MovementSpeed, EFFECT_SPEED_ID),
        Some(f64::from(0.2f32) * 2.0)
    );
    effect_attributes_changed(&mut m, "minecraft:speed", None);
    assert!(rows(&m).is_empty());
}

#[test]
fn an_enderman_adds_attacking_on_a_target_and_removes_the_id_on_none() {
    const ID: &str = "minecraft:attacking";
    let mut m = mob(Species::Enderman, "minecraft:enderman", 0.3);
    // A manual row with the id stays while it has a target (hasModifier guard).
    add_modifier(&mut m, Attr::MovementSpeed, ID, 0.5, Op::AddValue);
    m.set_target(Some(Who::Player));
    assert_eq!(rows(&m), [(ID.to_string(), 0.5, false)]);
    // setTarget(null) removes whatever holds the id, the manual row too.
    m.set_target(None);
    assert!(rows(&m).is_empty());
    // An unchanged None runs setTarget again: still nothing, no failure.
    m.set_target(None);
    assert!(rows(&m).is_empty());
    m.set_target(Some(Who::Player));
    m.set_target(Some(Who::Player));
    assert_eq!(rows(&m), [(ID.to_string(), f64::from(0.15f32), true)]);
    assert_eq!(
        f64::from(0.15f32).to_bits(),
        0.15000000596046448f64.to_bits()
    );
    // Another species keeps plain target semantics.
    let mut z = mob(Species::Zombie, "minecraft:zombie", 0.23);
    z.set_target(Some(Who::Player));
    assert!(rows(&z).is_empty());
}

#[test]
fn a_zombified_piglin_adds_attacking_while_angry_and_removes_it_when_calm() {
    const ID: &str = "minecraft:attacking";
    let mut m = mob(Species::ZombifiedPiglin, "minecraft:zombified_piglin", 0.23);
    m.extra.anger = 10;
    crate::mobs::undead::zombified_piglin_attacking_speed(&mut m);
    assert_eq!(rows(&m), [(ID.to_string(), 0.05, true)]);
    crate::mobs::undead::zombified_piglin_attacking_speed(&mut m);
    assert_eq!(rows(&m).len(), 1);
    // Becoming a baby while angry removes nothing.
    set_baby(&mut m, true);
    crate::mobs::undead::zombified_piglin_attacking_speed(&mut m);
    assert!(has_modifier(&m, Attr::MovementSpeed, ID));
    m.extra.anger = 0;
    crate::mobs::undead::zombified_piglin_attacking_speed(&mut m);
    assert!(!has_modifier(&m, Attr::MovementSpeed, ID));
    // An angry baby adds none.
    m.extra.anger = 10;
    crate::mobs::undead::zombified_piglin_attacking_speed(&mut m);
    assert!(!has_modifier(&m, Attr::MovementSpeed, ID));
    // A manual row stays while angry and goes once calm.
    let mut a = mob(Species::ZombifiedPiglin, "minecraft:zombified_piglin", 0.23);
    add_modifier(&mut a, Attr::MovementSpeed, ID, 0.5, Op::AddValue);
    a.extra.anger = 10;
    crate::mobs::undead::zombified_piglin_attacking_speed(&mut a);
    assert_eq!(rows(&a), [(ID.to_string(), 0.5, false)]);
    a.extra.anger = 0;
    crate::mobs::undead::zombified_piglin_attacking_speed(&mut a);
    assert!(rows(&a).is_empty());
}

#[test]
fn a_strider_replaces_a_manual_suffocating_row_and_keeps_it_saved() {
    const ID: &str = "minecraft:suffocating";
    let mut m = mob(Species::Strider, "minecraft:strider", 0.175);
    add_modifier(&mut m, Attr::MovementSpeed, ID, 0.1, Op::AddValue);
    crate::mobs::strider::set_suffocating(&mut m, true);
    crate::mobs::strider::set_suffocating(&mut m, true);
    assert_eq!(rows(&m), [(ID.to_string(), f64::from(-0.34f32), true)]);
    assert_eq!(saved(&m), [(ID.to_string(), 0.1)]);
    assert_eq!(
        f64::from(-0.34f32).to_bits(),
        (-0.3400000035762787f64).to_bits()
    );
    // addOrUpdateTransientModifier keeps the permanent row's ADD_VALUE bucket
    // entry: (0.175 + 0.1) + (0.175 + 0.1) * -0.34F.
    let added = 0.175 + 0.1;
    assert_eq!(m.movement_speed(), added + added * f64::from(-0.34f32));
    crate::mobs::strider::set_suffocating(&mut m, false);
    assert!(rows(&m).is_empty() && saved(&m).is_empty());
}

#[test]
fn sprinting_adds_one_row_and_a_remote_flag_adds_none() {
    let mut m = mob(Species::Cat, "minecraft:cat", 0.3);
    set_sprinting(&mut m, true);
    set_sprinting(&mut m, true);
    assert_eq!(
        rows(&m),
        [(SPRINTING_ID.to_string(), f64::from(0.3f32), true)]
    );
    set_sprinting(&mut m, false);
    assert!(rows(&m).is_empty());
    // A client's shared-flag update is a flag only (no modifier, no speed change).
    let mut remote = mob(Species::Cat, "minecraft:cat", 0.3);
    remote.extra.sprinting = true;
    assert!(rows(&remote).is_empty());
    assert_eq!(remote.movement_speed(), 0.3);
}

#[test]
fn set_baby_adds_the_transient_baby_row_and_a_removal_lasts_until_the_next() {
    let mut z = mob(Species::Zombie, "minecraft:zombie", 0.23);
    set_baby(&mut z, true);
    assert_eq!(rows(&z), [(BABY_SPEED_ID.to_string(), 0.5, true)]);
    assert!(saved(&z).is_empty());
    remove_modifier(&mut z, Attr::MovementSpeed, BABY_SPEED_ID);
    assert!(rows(&z).is_empty());
    set_baby(&mut z, true);
    assert_eq!(rows(&z).len(), 1);
    set_baby(&mut z, false);
    assert!(rows(&z).is_empty());
}

/// The separate baby-piglin consequence: `Piglin.SPEED_MODIFIER_BABY` is
/// 0.20000000298023224d ADD_MULTIPLIED_BASE (Piglin.<clinit>, setBaby), so a
/// baby piglin moves at its base times 1.2F-as-double, not 1.5.
#[test]
fn a_baby_piglin_moves_with_its_attribute_value() {
    let mut p = mob(Species::Piglin, "minecraft:piglin", 0.35);
    set_baby(&mut p, true);
    let row = modifier(&p, Attr::MovementSpeed, BABY_SPEED_ID).unwrap();
    assert_eq!(row.amount.to_bits(), 0.20000000298023224f64.to_bits());
    assert_eq!(p.movement_speed(), 0.35 + 0.35 * 0.20000000298023224);
}

#[test]
fn slowness_applies_once_with_its_float_amount_and_the_end_clamp() {
    let mut m = mob(Species::Pig, "minecraft:pig", 0.25);
    m.extra.effects.push(("minecraft:slowness", -1, 0));
    assert_eq!(m.movement_speed(), 0.25 * (1.0 + -0.15000000596046448));
    // Slowness VII: 1 - 1.05..., below zero, clamped once at the end to 0.
    m.extra.effects[0].2 = 6;
    assert_eq!(m.movement_speed(), 0.0);
}

/// Java's per-operation iteration order (fastutil slots, top down): sprinting
/// sits at slot 23, effect.speed at 19, effect.slowness at 2, so their
/// ADD_MULTIPLIED_TOTAL factors multiply in that order.
#[test]
fn rows_of_one_operation_multiply_in_javas_slot_order() {
    let mut m = mob(Species::Cat, "minecraft:cat", 0.3);
    m.extra.effects.push(("minecraft:slowness", -1, 0));
    m.add_effect("minecraft:speed", -1, 1);
    set_sprinting(&mut m, true);
    assert_eq!(
        bucket(&m, Operation::AddMultipliedTotal)
            .into_iter()
            .map(|x| x.0)
            .collect::<Vec<_>>(),
        ["minecraft:sprinting", "minecraft:effect.speed"]
    );
    let expected = 0.3
        * (1.0 + f64::from(0.3f32))
        * (1.0 + f64::from(0.2f32) * 2.0)
        * (1.0 + f64::from(-0.15f32));
    assert_eq!(m.movement_speed().to_bits(), expected.to_bits());
}

/// The removal routes take the Speed row away only on a had-to-gone change:
/// a death_protection ClearAllEffects / RemoveEffects (`onEffectsRemoved`).
#[test]
fn consume_effect_removals_take_the_speed_row_away() {
    use crate::items::ConsumeEffect;
    let mut m = mob(Species::Pig, "minecraft:pig", 0.25);
    m.add_effect("minecraft:speed", -1, 1);
    add_modifier(&mut m, Attr::MovementSpeed, "t:keep", 0.01, Op::AddValue);
    m.apply_death_effect(&ConsumeEffect::RemoveEffects(vec![
        "minecraft:poison".into(),
    ]));
    assert!(has_modifier(&m, Attr::MovementSpeed, EFFECT_SPEED_ID));
    m.apply_death_effect(&ConsumeEffect::RemoveEffects(vec![
        "minecraft:speed".into(),
    ]));
    assert!(!has_modifier(&m, Attr::MovementSpeed, EFFECT_SPEED_ID));
    m.add_effect("minecraft:speed", -1, 0);
    m.apply_death_effect(&ConsumeEffect::ClearAllEffects);
    assert!(!has_modifier(&m, Attr::MovementSpeed, EFFECT_SPEED_ID));
    // An unrelated row stays.
    assert!(has_modifier(&m, Attr::MovementSpeed, "t:keep"));
}

/// `ConversionType.convertCommon`: `newMob.addEffect` for each effect, so the
/// converted mob gets a fresh Speed row; nothing else is copied.
#[test]
fn a_conversion_gives_a_copied_speed_effect_its_row() {
    let mut from = mob(Species::Zombie, "minecraft:zombie", 0.23);
    from.add_effect("minecraft:speed", -1, 1);
    let mut to = mob(Species::Drowned, "minecraft:drowned", 0.23);
    crate::mobs::conversion::single(&from, &mut to, crate::mobs::conversion::Params::KEEP_ALL);
    assert_eq!(
        modifier_amount(&to, Attr::MovementSpeed, EFFECT_SPEED_ID),
        Some(f64::from(0.2f32) * 2.0)
    );
    assert_eq!(rows(&to).len(), 1);
}

/// `AbstractHorse.setOffspringAttribute`: the parents' base values, so a
/// parent's Speed row does not change the foal (same draws, same speed).
#[test]
fn a_foal_draws_its_speed_from_the_parents_base_values() {
    use horizon_core::random::LegacyRandom;
    let horse = |speed: bool| {
        let mut h = mob(Species::Horse, "minecraft:horse", 0.225);
        if speed {
            h.add_effect("minecraft:speed", -1, 1);
        }
        h
    };
    let foal = |speed: bool| {
        let (a, b) = (horse(speed), horse(speed));
        let mut child = mob(Species::Horse, "minecraft:horse", 0.225);
        crate::mobs::equine::inherit(&mut child, &a, &b, &mut LegacyRandom::new(5));
        base_value(&child, Attr::MovementSpeed)
    };
    assert_eq!(foal(true).to_bits(), foal(false).to_bits());
}

/// `PhantomMoveControl.speed` is the control's own float: an attribute
/// command neither sets nor reads it.
#[test]
fn a_phantoms_control_speed_is_not_the_movement_attribute() {
    let mut m = mob(Species::Phantom, "minecraft:phantom", 0.7);
    crate::mobs::phantom::register(&mut m);
    assert_eq!(m.ai.mov.phantom_speed, 0.1);
    add_modifier(&mut m, Attr::MovementSpeed, "t:x", 0.5, Op::AddValue);
    set_base(&mut m, Attr::MovementSpeed, 2.0);
    assert_eq!(m.ai.mov.phantom_speed, 0.1);
}

/// Required case: the Speed row and a legal manual id that collides in the
/// same ADD_MULTIPLIED_TOTAL slot. `minecraft:effect.speed` and `t:s0` both
/// hash to slot 19 of the 32 (`Identifier.hashCode`, `HashCommon.mix`). Both
/// rows live in the persistent instance, so the order is the map's own
/// history: the later id probes to slot 20 and is iterated first; removing
/// Speed shifts `t:s0` back to 19 (`shiftKeys`), so Speed added again sits at
/// 20 and now comes first. With base 0.3 and 0.05 the two orders differ in the
/// last bit, so the bits prove the order.
#[test]
fn speed_and_a_colliding_manual_id_follow_the_maps_history() {
    const MANUAL: &str = "t:s0";
    let speed_ii: f64 = 2.0 * 0.20000000298023224;
    let manual_first = 0.3 * (1.0 + 0.05) * (1.0 + speed_ii);
    let speed_first = 0.3 * (1.0 + speed_ii) * (1.0 + 0.05);
    assert_ne!(manual_first.to_bits(), speed_first.to_bits());
    let ids = |m: &Mob| -> Vec<String> {
        bucket(m, Operation::AddMultipliedTotal)
            .into_iter()
            .map(|x| x.0)
            .collect()
    };
    let mut m = mob(Species::Pig, "minecraft:pig", 0.3);
    // Speed first (slot 19), then the manual row (probes to 20).
    m.add_effect("minecraft:speed", -1, 1);
    add_modifier(
        &mut m,
        Attr::MovementSpeed,
        MANUAL,
        0.05,
        Op::AddMultipliedTotal,
    );
    assert_eq!(ids(&m), [MANUAL, EFFECT_SPEED_ID]);
    assert_eq!(m.movement_speed().to_bits(), manual_first.to_bits());
    // Speed ends (the effect leaves the list, then onEffectsRemoved): t:s0
    // shifts back to slot 19.
    m.extra.effects.retain(|e| e.0 != "minecraft:speed");
    effect_attributes_changed(&mut m, "minecraft:speed", None);
    assert_eq!(ids(&m), [MANUAL]);
    assert_eq!(m.movement_speed(), 0.3 * (1.0 + 0.05));
    // Speed again: it probes to slot 20 and is now iterated first.
    m.add_effect("minecraft:speed", -1, 1);
    assert_eq!(ids(&m), [EFFECT_SPEED_ID, MANUAL]);
    assert_eq!(m.movement_speed().to_bits(), speed_first.to_bits());
    // A stronger Speed is remove + add (addOrReplacePermanentModifier): the
    // slot stays 20 because 19 is still taken.
    m.add_effect("minecraft:speed", -1, 2);
    assert_eq!(ids(&m), [EFFECT_SPEED_ID, MANUAL]);
    // The manual row's removal shifts nothing below it; Speed stays alone.
    remove_modifier(&mut m, Attr::MovementSpeed, MANUAL);
    assert_eq!(ids(&m), [EFFECT_SPEED_ID]);
}
