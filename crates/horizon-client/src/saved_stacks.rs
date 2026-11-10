//! Typed snapshots of real block-entity stacks. Raw template payloads are opaque.
use crate::{containers::BlockEntity, items::Items};
use horizon_items::saved_stack::{capture, capture_slots};

pub fn capture_block_entity(entity: &mut BlockEntity, items: &Items) {
    capture_slots(entity.slots_mut(), items);
    match entity {
        BlockEntity::Lectern(book) => {
            book.book = book.book.as_ref().map(|s| capture(s, items));
        }
        BlockEntity::Brushable(brush) => {
            brush.item = brush.item.as_ref().map(|s| capture(s, items));
        }
        BlockEntity::Campfire(fire) => capture_slots(&mut fire.items, items),
        BlockEntity::Vault(vault) => {
            vault.config.key_item = vault.config.key_item.as_ref().map(|s| capture(s, items));
            vault.display_item = vault.display_item.as_ref().map(|s| capture(s, items));
            for stack in &mut vault.items_to_eject {
                *stack = capture(stack, items);
            }
        }
        _ => {}
    }
}

pub fn block_entity_tag(
    pos: (i32, i32, i32),
    entity: &BlockEntity,
    name: &str,
    items: &Items,
) -> horizon_world::nbt::Tag {
    let mut captured = entity.clone();
    capture_block_entity(&mut captured, items);
    crate::save::encode_block_entity(pos, &captured, name)
}

/// Capture only Level's modelled actual stacks; player_extra remains opaque.
pub fn capture_level(level: &mut crate::save::Level, items: &Items) {
    for (slot, id, count, damage) in &level.inventory {
        let stack = crate::items::ItemStack {
            item: id.as_str().into(),
            count: *count,
            damage: *damage,
            extra: level.inventory_components.get(slot).cloned(),
        };
        match capture(&stack, items).extra {
            Some(extra) => {
                level.inventory_components.insert(*slot, extra);
            }
            None => {
                level.inventory_components.remove(slot);
            }
        }
    }
    capture_slots(&mut level.ender_items, items);
}

/// DecoratedPotBlockEntity's live item uses ItemStack.CODEC even though this
/// class is kept in ChunkExtras. Its sherds are passive ItemStackTemplates.
/// This is an explicit class/field codec boundary, never recursive tag guessing.
pub fn capture_pot_item(tag: &mut horizon_world::nbt::Tag, items: &Items) {
    use horizon_world::nbt::Tag;
    if crate::block_entity_nbt::id(tag).as_deref() != Some("minecraft:decorated_pot") {
        return;
    }
    if let Tag::Compound(fields) = tag
        && let Some(stack) = fields.get_mut(&"item".into())
    {
        capture_stack_tag(stack, items);
    }
}

/// The roll is an actual ContainerHelper capture. Its comparison witness must
/// use the same saved patch as the current container, without changing the
/// live roll or the policy deciding whether that roll is still untouched.
pub fn capture_rolled_loot(roll: &mut crate::chunk::RolledLoot, items: &Items) {
    if let horizon_world::nbt::Tag::List(stacks) = &mut roll.items {
        for stack in stacks {
            capture_stack_tag(stack, items);
        }
    }
}

// Only explicit actual-stack fields reach this helper. Decode for prototype
// authority, then remove only the qualifying key from the original capture:
// re-encoding the whole tag could also change count, Slot or unrelated fields.
fn capture_stack_tag(tag: &mut horizon_world::nbt::Tag, items: &Items) {
    use horizon_world::nbt::Tag;
    let Some(stack) = crate::save::item_from_nbt(tag) else {
        return;
    };
    let saved = capture(&stack, items);
    if stack
        .data()
        .is_none_or(|data| data.other("minecraft:max_stack_size").is_none())
        || saved
            .data()
            .is_some_and(|data| data.other("minecraft:max_stack_size").is_some())
    {
        return;
    }
    if let Tag::Compound(fields) = tag
        && let Some(Tag::Compound(patch)) = fields.get_mut(&"components".into())
        && patch.remove(&"minecraft:max_stack_size".into()).is_some()
        && patch.is_empty()
    {
        fields.remove(&"components".into());
    }
}

#[cfg(test)]
mod tests;
