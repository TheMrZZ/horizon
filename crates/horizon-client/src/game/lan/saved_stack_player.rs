//! Catalog-dependent saved stack projection at its existing caller.
use super::*;
impl Game<'_> {
    /// Save a guest when they leave or the world closes (after
    /// `take_parent_vehicle`, `PlayerList.save`): an integrated server's owner
    /// into the level (`/data`) and both its files (`playerdata::save_owner`),
    /// anyone else into their own file.
    pub(in crate::game) fn save_guest(
        &mut self,
        host: &LanHost,
        guest: &mut horizon_client::lan::guest::Guest,
    ) {
        self.guest_attachments_into(guest);
        // A player a proxy moved to another server (`player-release moved`) is saved
        // there now, not here.
        if guest.placed && !guest.released {
            let _ = self.finish_player_autosaves(true);
            if host.owner.as_ref().is_some_and(|o| o.uuid == guest.uuid) {
                playerdata::save_owner_with_items(
                    guest,
                    &mut self.level,
                    &self.world_root,
                    self.world_tables
                        .as_ref()
                        .map_or(self.res.items(), |tables| &tables.items),
                );
            } else {
                self.player_autosaves.save_after_with_items(
                    guest,
                    &self.world_root,
                    self.world_tables
                        .as_ref()
                        .map_or(self.res.items(), |tables| &tables.items),
                );
            }
            let directory = playerdata::directory(&self.world_root);
            self.native_player_save_to_file(guest.uuid, &guest.name, &directory);
        }
        self.save_guest_records(host, guest);
    }
}
