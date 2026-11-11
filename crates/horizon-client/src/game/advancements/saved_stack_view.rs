//! Catalog-dependent saved stack projection at its existing caller.
use super::*;
impl Game<'_> {
    /// `Entity.get`: the mob's live custom-name component and type variants,
    /// alongside the other state the entity predicate asks for.
    pub(in crate::game) fn mob_entity(&self, m: &horizon_client::mobs::Mob) -> EntityView {
        // mainhand, offhand, head, chest, legs, feet.
        let slot = |i: &Option<std::sync::Arc<str>>| {
            i.as_ref().map(|id| item_view(&ItemStack::new(id, 1)))
        };
        let mut equipment: [Option<ItemView>; 7] = Default::default();
        equipment[0] = slot(&m.extra.main_hand);
        equipment[1] = slot(&m.extra.off_hand);
        equipment[2] = slot(&m.extra.head);
        equipment[3] = slot(&m.extra.chest);
        equipment[4] = slot(&m.extra.legs);
        equipment[5] = slot(&m.extra.feet);
        // The body slot (a wolf's armor, a horse's), with its durability.
        equipment[6] = m.extra.body.as_ref().map(|id| {
            let mut armor = ItemStack::new(id, 1);
            armor.damage = m.extra.body_damage.max(0) as u32;
            item_view(&armor)
        });
        // Only the new snapshot is adopted; the existing view fields stay as
        // constructed above. Body and other view consumers stay unprojected.
        use horizon_client::mobs::finalize;
        for (view, slot) in equipment[..6].iter_mut().zip([
            finalize::MAINHAND,
            finalize::OFFHAND,
            finalize::HEAD,
            finalize::CHEST,
            finalize::LEGS,
            finalize::FEET,
        ]) {
            if let Some(view) = view {
                let data = finalize::item_stack(m, slot)
                    .and_then(|stack| stack.custom_data().cloned())
                    .unwrap_or_default();
                view.custom_data = Some(horizon_world::nbt::Tag::Compound(data));
            }
        }
        let passengers: Vec<EntityView> = self
            .mobs
            .iter()
            .filter(|r| r.extra.mount == Some(m.id))
            .map(|r| self.mob_entity(r))
            .chain(self.rider_entity(m.id))
            .collect();
        // `NbtPredicate.getEntityTagToCompare`: the mob as saved, with the
        // mobs riding it under `Passengers`, when a criterion may ask.
        let nbt = self.adv.tree.tests_nbt().then(|| {
            let uuid = self.persist.uuids.get(&m.id).copied().unwrap_or_default();
            let raw = self.persist.raw.get(&m.id);
            let registry = Some(self.pack.assets.registry);
            let mut tag = horizon_client::entity_nbt::mob_to_nbt_with_items(
                m,
                uuid,
                raw,
                registry,
                |other| self.persist.uuid_of(other),
                self.items(),
            );
            self.put_last_hurt_by_mob(m, &mut tag);
            let riders = passengers.iter().filter_map(|r| r.nbt.clone()).collect();
            crate::game::persist::add_passengers(&mut tag, riders);
            tag
        });
        let movement = (m.extra.mount.is_none()
            && m.rider.is_none()
            && self.riding != Some(m.id)
            && self.lan.as_ref().is_none_or(|host| {
                host.hub
                    .guests
                    .iter()
                    .all(|guest| guest.vehicle != Some(m.id))
            }))
        .then_some(adv::MovementView {
            per_tick: [m.velocity.x, m.velocity.y, m.velocity.z],
            fall_distance: m.fall_distance,
        });
        EntityView {
            movement,
            kind: m.kind.id.clone(),
            position: [m.position.x, m.position.y, m.position.z],
            on_fire: m.fire_ticks > 0,
            baby: m.is_baby(),
            on_ground: m.on_ground,
            equipment,
            // LivingEntity.getActiveEffectsMap supplies the active predicate instances.
            effects: m
                .extra
                .effects
                .iter()
                .map(|&(id, duration, amplifier)| {
                    let id = horizon_client::brewing::namespaced(id);
                    (id.to_owned(), amplifier, duration)
                })
                .collect(),
            components: horizon_client::mobs::finalize::variant_component(m)
                .map(|(id, value)| (id.to_owned(), value))
                .into_iter()
                .collect(),
            custom_name: m.custom_name_text.as_ref().map(Text::component),
            passengers,
            nbt,
            team: self.mob_team_of(m).map(|t| t.name.clone()),
            ..Default::default()
        }
    }
}
