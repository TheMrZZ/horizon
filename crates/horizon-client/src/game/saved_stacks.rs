//! Capture catalog-dependent stack patches before the asynchronous save boundary.
use super::*;
use horizon_items::saved_stack::{capture, capture_slots};

impl Game<'_> {
    pub(super) fn capture_player_stacks(&mut self) {
        let items = self.items();
        let components = self
            .inventory
            .iter()
            .enumerate()
            .filter_map(|(i, stack)| Some((i as u8, capture(stack.as_ref()?, items).extra?)))
            .chain(self.armor.iter().enumerate().filter_map(|(i, stack)| {
                Some((103 - i as u8, capture(stack.as_ref()?, items).extra?))
            }))
            .chain(
                self.offhand
                    .as_ref()
                    .and_then(|stack| Some((150, capture(stack, items).extra?))),
            )
            .collect();
        let mut ender = self.ender_items.clone();
        capture_slots(&mut ender, items);
        self.level.inventory_components = components;
        self.level.ender_items = ender;
    }

    pub(super) fn with_entities<'a>(
        &self,
        chunks: impl IntoIterator<Item = &'a Arc<horizon_client::chunk::Chunk>>,
    ) -> Vec<Arc<horizon_client::chunk::Chunk>> {
        horizon_client::save::chunks_with_block_entities(
            chunks,
            &self.world.block_entities,
            &self.world.block_names,
        )
        .into_iter()
        .map(|mut chunk| {
            for (_, entity) in &mut chunk.block_entities {
                horizon_client::saved_stacks::capture_block_entity(entity, self.items());
            }
            self.attach_save_data(&mut chunk);
            if let Some(saved) = &mut chunk.saved {
                let saved = Arc::make_mut(saved);
                for source in &mut saved.block_entities {
                    horizon_client::saved_stacks::capture_pot_item(source, self.items());
                }
                for roll in &mut saved.rolled_loot {
                    horizon_client::saved_stacks::capture_rolled_loot(roll, self.items());
                }
            }
            Arc::new(chunk)
        })
        .collect()
    }
}
