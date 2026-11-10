//! Whole body equipment roots; rendering keeps the existing item-id field.
use super::*;
impl Mob {
    /// Moving an equipped root retains its patch; an id-only source still creates its bare stack.
    pub fn take_body_stack(&mut self) -> Option<crate::items::ItemStack> {
        let stack = self.equipment_stacks()[6].clone();
        self.extra.set_body_stack(None);
        stack
    }
    /// What the mob holds and wears, by `EquipmentSlot` ordinal: main hand,
    /// off hand, feet, legs, chest, head, body and saddle
    /// (`ClientboundSetEquipmentPacket`'s slots).
    pub fn equipment_stacks(&self) -> [Option<crate::items::ItemStack>; 8] {
        use crate::items::ItemStack;
        let e = &self.extra;
        let wolf = self.species() == Species::Wolf;
        let slots = [
            (0usize, &e.main_hand, e.main_hand_stack()),
            (1, &e.off_hand, None),
            (2, &e.feet, e.armor[3].as_ref()),
            (3, &e.legs, e.armor[2].as_ref()),
            (4, &e.chest, e.armor[1].as_ref()),
            // The body slot's item is no worn armour: a horse's armour, a
            // llama's carpet, a harness go out on their own (6).
            (5usize, &e.head, e.armor[0].as_ref()),
            (6, &e.body, None),
        ];
        let mut out: [Option<ItemStack>; 8] = Default::default();
        for (slot, item, worn) in slots {
            let Some(item) = item else {
                continue;
            };
            // The worn armour keeps its stack: a dyed leather piece keeps
            // its colour.
            let worn = worn.cloned().or_else(|| {
                (slot == 6)
                    .then(|| e.body_stack.clone())
                    .flatten()
                    .filter(|s| s.item == *item)
            });
            out[slot] = Some(worn.unwrap_or_else(|| ItemStack {
                item: item.clone(),
                count: 1,
                // A wolf's armor shows its cracks.
                damage: if slot == 6 && wolf {
                    e.body_damage.max(0) as u32
                } else {
                    0
                },
                extra: if slot == 6 && wolf {
                    e.body_extra.clone()
                } else {
                    None
                },
            }));
        }
        if wolf && let Some(body) = &mut out[6] {
            body.damage = e.body_damage.max(0) as u32;
            body.extra = e.body_extra.clone();
        }
        if e.saddled {
            out[7] = Some(e.saddle_stack.clone().unwrap_or(ItemStack {
                item: Arc::from("minecraft:saddle"),
                count: 1,
                damage: 0,
                extra: None,
            }));
        }
        out
    }
}
impl Extra {
    pub fn set_body_stack(&mut self, stack: Option<crate::items::ItemStack>) {
        self.body = stack.as_ref().map(|s| s.item.clone());
        self.body_stack = stack;
    }
}
