//! Validate the slot-source candidate graph before replacing the loaded snapshot.
use super::{DataStore, Game};
use horizon_client::command::args::Registries;
impl Game<'_> {
    pub(in crate::game) fn slot_source_registries(&self, store: &DataStore) -> Result<Registries, String> {
        if let Some((id, error)) = store.slot_source_errors.first() {
            return Err(format!("{id}: {error}"));
        }
        let mut registries = self.ui.registries.clone();
        let entry = registries
            .by_registry
            .entry("minecraft:slot_source".into())
            .or_default();
        entry.ids = store.slot_sources.keys().cloned().collect();
        entry.tags = store
            .slot_source_tags
            .keys()
            .filter(|id| store.slot_source_tag(id).is_ok())
            .cloned()
            .collect();
        for (id, value) in &store.slot_sources {
            let (tag, _) = horizon_core::nbt::parse_snbt(&value.to_string())
                .map_err(|error| format!("{id}: {error}"))?;
            // RegistryDataLoader reads a slot_source file with
            // SlotSources.TYPED_CODEC: only an object with a `type` loads, not a
            // holder id or an inline list (which the holder codec would accept).
            if !value.is_object() {
                return Err(format!(
                    "{id}: {}",
                    horizon_client::command::codec::not_map(&tag)
                ));
            }
            horizon_client::command::codec::decode("minecraft:slot_source", &tag, &registries)
                .map_err(|error| format!("{id}: {error}"))?;
            super::super::commands::items::slot_sources::expand(
                store,
                &serde_json::Value::String(id.clone()),
            )?;
        }
        Ok(registries)
    }
}
