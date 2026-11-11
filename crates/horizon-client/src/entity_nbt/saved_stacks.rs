//! Borrowed authority for actual saved stack roots, never raw template recursion.
use super::*;
#[derive(Clone, Copy)]
pub(super) struct SavedStacks<'a>(pub Option<&'a crate::items::Items>);
impl SavedStacks<'_> {
    pub(super) fn stack(self, stack: &crate::items::ItemStack) -> Tag {
        match self.0 {
            Some(items) => horizon_items::item_nbt::item_to_nbt_with_items(stack, items),
            None => item_to_nbt(stack),
        }
    }
    pub(super) fn slots(self, slots: &[Option<crate::items::ItemStack>]) -> Tag {
        let mut slots = slots.to_vec();
        if let Some(items) = self.0 {
            horizon_items::saved_stack::capture_slots(&mut slots, items);
        }
        crate::save::encode_items(&slots)
    }
}
pub fn mob_to_nbt(
    m: &Mob,
    uuid: [i32; 4],
    raw: Option<&Tag>,
    registry: Option<&BlockRegistry>,
    uuid_of: impl FnMut(u32) -> Option<[i32; 4]>,
) -> Tag {
    super::mob_to_nbt_saved(m, uuid, raw, registry, uuid_of, None)
}

pub fn mob_to_nbt_with_items(
    m: &Mob,
    uuid: [i32; 4],
    raw: Option<&Tag>,
    registry: Option<&BlockRegistry>,
    uuid_of: impl FnMut(u32) -> Option<[i32; 4]>,
    catalog: &crate::items::Items,
) -> Tag {
    super::mob_to_nbt_saved(m, uuid, raw, registry, uuid_of, Some(catalog))
}

pub fn ominous_item_spawner_to_nbt(
    item: &crate::items::ItemStack,
    position: Vec3,
    spawn_item_after: i64,
    uuid: [i32; 4],
    raw: Option<&Tag>,
) -> Tag {
    super::ominous_item_spawner_to_nbt_saved(item, position, spawn_item_after, uuid, raw, None)
}

pub fn ominous_item_spawner_to_nbt_with_items(
    item: &crate::items::ItemStack,
    position: Vec3,
    spawn_item_after: i64,
    uuid: [i32; 4],
    raw: Option<&Tag>,
    catalog: &crate::items::Items,
) -> Tag {
    super::ominous_item_spawner_to_nbt_saved(
        item,
        position,
        spawn_item_after,
        uuid,
        raw,
        Some(catalog),
    )
}

pub fn arrow_to_nbt(
    p: &Projectile,
    uuid: [i32; 4],
    owner: Option<[i32; 4]>,
    in_block: Option<Tag>,
    raw: Option<&Tag>,
) -> Option<Tag> {
    super::arrow_to_nbt_saved(p, uuid, owner, in_block, raw, None)
}

pub fn arrow_to_nbt_with_items(
    p: &Projectile,
    uuid: [i32; 4],
    owner: Option<[i32; 4]>,
    in_block: Option<Tag>,
    raw: Option<&Tag>,
    catalog: &crate::items::Items,
) -> Option<Tag> {
    super::arrow_to_nbt_saved(p, uuid, owner, in_block, raw, Some(catalog))
}

pub fn fireball_to_nbt(
    p: &Projectile,
    uuid: [i32; 4],
    owner: Option<[i32; 4]>,
    raw: Option<&Tag>,
) -> Option<Tag> {
    super::fireball_to_nbt_saved(p, uuid, owner, raw, None)
}

pub fn fireball_to_nbt_with_items(
    p: &Projectile,
    uuid: [i32; 4],
    owner: Option<[i32; 4]>,
    raw: Option<&Tag>,
    catalog: &crate::items::Items,
) -> Option<Tag> {
    super::fireball_to_nbt_saved(p, uuid, owner, raw, Some(catalog))
}

pub fn throwable_to_nbt(
    p: &Projectile,
    uuid: [i32; 4],
    owner: Option<[i32; 4]>,
    raw: Option<&Tag>,
) -> Option<Tag> {
    super::throwable_to_nbt_saved(p, uuid, owner, raw, None)
}

pub fn throwable_to_nbt_with_items(
    p: &Projectile,
    uuid: [i32; 4],
    owner: Option<[i32; 4]>,
    raw: Option<&Tag>,
    catalog: &crate::items::Items,
) -> Option<Tag> {
    super::throwable_to_nbt_saved(p, uuid, owner, raw, Some(catalog))
}

pub fn firework_rocket_to_nbt(r: &RocketSave, uuid: [i32; 4], raw: Option<&Tag>) -> Tag {
    super::firework_rocket_to_nbt_saved(r, uuid, raw, None)
}

pub fn firework_rocket_to_nbt_with_items(
    r: &RocketSave,
    uuid: [i32; 4],
    raw: Option<&Tag>,
    catalog: &crate::items::Items,
) -> Tag {
    super::firework_rocket_to_nbt_saved(r, uuid, raw, Some(catalog))
}

pub fn armor_stand_to_nbt(a: &ArmorStand, uuid: [i32; 4], raw: Option<&Tag>) -> Tag {
    super::armor_stand_to_nbt_saved(a, uuid, raw, None)
}

pub fn armor_stand_to_nbt_with_items(
    a: &ArmorStand,
    uuid: [i32; 4],
    raw: Option<&Tag>,
    catalog: &crate::items::Items,
) -> Tag {
    super::armor_stand_to_nbt_saved(a, uuid, raw, Some(catalog))
}

pub fn boat_to_nbt(
    id: &str,
    position: Vec3,
    motion: Vec3,
    rotation: [f32; 2],
    ground: (bool, f64),
    uuid: [i32; 4],
    raw: Option<&Tag>,
    items: Option<&[Option<crate::items::ItemStack>]>,
) -> Tag {
    super::boat_to_nbt_saved(
        id, position, motion, rotation, ground, uuid, raw, items, None,
    )
}

pub fn boat_to_nbt_with_items(
    id: &str,
    position: Vec3,
    motion: Vec3,
    rotation: [f32; 2],
    ground: (bool, f64),
    uuid: [i32; 4],
    raw: Option<&Tag>,
    items: Option<&[Option<crate::items::ItemStack>]>,
    catalog: &crate::items::Items,
) -> Tag {
    super::boat_to_nbt_saved(
        id,
        position,
        motion,
        rotation,
        ground,
        uuid,
        raw,
        items,
        Some(catalog),
    )
}
