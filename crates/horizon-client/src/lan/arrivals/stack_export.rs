//! Player export captures the selected catalog before producing immutable transfer bytes.
use super::*;
/// The record of the player at guest index `i` (which [`check_exportable`] gave,
/// with its menus closed and its attachments in place): the player compound as
/// the server saves it, as the uncompressed NBT of the compound in base64, its
/// advancements and statistics, and the few values a proxy reads without opening
/// it. With `freeze`, a hand-over starts: the record gets an id, the player's
/// own inputs stop being acted on until the hand-over ends, and every later
/// export answers with this same record. A player already frozen is answered
/// with its hand-over's record, whatever `freeze` says.
pub fn export_record(
    hub: &mut Hub,
    i: usize,
    freeze: bool,
    records: Records,
) -> Result<serde_json::Value, String> {
    export_record_saved(hub, i, freeze, records, None)
}
pub fn export_record_with_items(
    hub: &mut Hub,
    i: usize,
    freeze: bool,
    records: Records,
    catalog: &crate::items::Items,
) -> Result<serde_json::Value, String> {
    export_record_saved(hub, i, freeze, records, Some(catalog))
}
fn export_record_saved(
    hub: &mut Hub,
    i: usize,
    freeze: bool,
    records: Records,
    catalog: Option<&crate::items::Items>,
) -> Result<serde_json::Value, String> {
    use horizon_world::nbt::{Limits, NamedTag, write_named};
    if let Some(h) = &hub.guests[i].handover {
        return Ok(h.record.clone());
    }
    let id = freeze.then(new_handover_id).transpose()?;
    let g = &hub.guests[i];
    let tag = match catalog {
        Some(items) => crate::lan::playerdata::player_tag_with_items(g, items),
        None => crate::lan::playerdata::player_tag(g),
    };
    let bytes = write_named(
        &NamedTag {
            name: "".into(),
            tag,
        },
        Limits::saved(),
    )
    .map_err(|e| e.to_string())?;
    let velocity = g
        .motion
        .as_ref()
        .map_or([0.0; 3], |m| [m.velocity.x, m.velocity.y, m.velocity.z]);
    let record = serde_json::json!({
        "version": RECORD_VERSION,
        "data_version": horizon_world::DATA_VERSION,
        "handover": id.map(handover_text),
        "uuid": horizon_protocol::crypto::uuid_string(g.uuid),
        "name": g.name,
        "dimension": g.dimension,
        "position": g.position,
        "rotation": [g.yaw, g.pitch],
        "velocity": velocity,
        "game_mode": g.game_mode,
        "player": crate::lan::base64(&bytes),
        "advancements": records.advancements,
        "stats": records.stats,
    });
    if let Some(id) = id {
        let since = hub.ticks;
        hub.guests[i].handover = Some(Handover {
            id,
            since,
            record: record.clone(),
        });
    }
    Ok(record)
}
