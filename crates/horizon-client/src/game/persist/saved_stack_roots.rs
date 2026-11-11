//! Catalog-aware snapshots of actual vehicle, stand and trident roots.
use super::*;
impl Game<'_> {
    pub(in crate::game) fn save_trident(
        &mut self,
        i: usize,
        keys: &HashSet<ChunkKey>,
    ) -> Option<(ChunkKey, Tag)> {
        let (id, at) = (self.tridents[i].id, self.tridents[i].position);
        keys.contains(&chunk_of(at)).then(|| {
            let (uuid, reg) = (self.uuid_for(id), self.pack.assets.registry);
            let tag = self.tridents[i].to_nbt_with_items(
                uuid,
                reg,
                self.persist.raw.get(&id),
                self.items(),
            );
            (chunk_of(at), tag)
        })
    }
    pub(in crate::game) fn save_boat(
        &mut self,
        i: usize,
        keys: &HashSet<ChunkKey>,
    ) -> Option<(ChunkKey, Tag)> {
        let b = &self.boats[i];
        let (id, at) = (b.id, b.position);
        if !keys.contains(&chunk_of(at)) {
            return None;
        }
        let uuid = self.saved_uuid_or_random(id);
        let mut tag = b.to_nbt_with_items(uuid, self.persist.raw.get(&id), self.items());
        self.put_mob_passengers(&mut tag, id);
        // AbstractBoat.addAdditionalSaveData: the lead's holder.
        self.write_boat_leash(&mut tag, i);
        Some((chunk_of(at), tag))
    }
    pub(in crate::game) fn save_cart(
        &mut self,
        i: usize,
        keys: &HashSet<ChunkKey>,
    ) -> Option<(ChunkKey, Tag)> {
        let (id, at) = (self.carts[i].id, self.carts[i].position);
        if !keys.contains(&chunk_of(at)) {
            return None;
        }
        let uuid = self.saved_uuid_or_random(id);
        let mut tag =
            self.carts[i].to_nbt_with_items(uuid, self.persist.raw.get(&id), self.items());
        self.put_mob_passengers(&mut tag, id);
        Some((chunk_of(at), tag))
    }
    pub(in crate::game) fn save_armor_stand(
        &mut self,
        i: usize,
        keys: &HashSet<ChunkKey>,
    ) -> Option<(ChunkKey, Tag)> {
        let a = &self.armor_stands[i];
        keys.contains(&chunk_of(a.position)).then(|| {
            let uuid = self.saved_uuid_or_random(a.id);
            let tag = entity_nbt::armor_stand_to_nbt_with_items(
                a,
                uuid,
                self.persist.raw.get(&a.id),
                self.items(),
            );
            (chunk_of(a.position), tag)
        })
    }
}
