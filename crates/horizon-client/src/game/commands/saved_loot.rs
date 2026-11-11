//! Catalog-consistent snapshot view of an unopened container loot roll.
use super::Game;

impl Game<'_> {
    /// The loot table and seed a container still holding its roll saves
    /// (`RolledLoot::restore`), in place of its items.
    pub(in crate::game) fn restore_loot_table(
        &self,
        key: (i32, i32, i32),
        tag: &mut horizon_world::nbt::Tag,
    ) {
        let horizon_world::nbt::Tag::Compound(map) = tag else {
            return;
        };
        let rolled = self
            .world
            .chunks
            .get(&(key.0 >> 4, key.2 >> 4))
            .and_then(|c| c.saved.as_ref())
            .and_then(|s| s.rolled_loot.iter().find(|r| r.pos == key));
        if let Some(r) = rolled {
            let mut saved = r.clone();
            horizon_client::saved_stacks::capture_rolled_loot(&mut saved, self.items());
            saved.restore(map);
        }
    }
}
