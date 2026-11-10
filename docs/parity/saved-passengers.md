# Saved passenger trees (26.3)

This completes the live-tree and loader breadth left by the partial
[#11831](https://github.com/horizon-rust/horizon/pull/11831) repair of
[#11822](https://github.com/horizon-rust/horizon/issues/11822).

## Required behavior and implementation

| Requirement | Shared production path and qualification |
| --- | --- |
| Unknown/missing IDs and non-compound entries create no saved rider | Registered-type admission and the existing per-kind loaders; original invalid-passenger regression retained |
| Valid non-mob riders are actual entities, recursively and in input order | All root loaders extract `Passengers` at the common load boundary; `PassengerSlot::Live` resolves actual loaded IDs through `level_entity` |
| Raw-root loaders also load their children | Armor stand, display, item and other registered roots use the same boundary as mobs, boats, carts and TNT |
| Null creation differs from no-save and refused boarding | Player has a null factory; fishing bobber, leash knot and lightning bolt are creatable but have no saved type ID; Marker rejects boarding |
| Saved/data-get trees contain current riders once | Existing live per-kind NBT writers, ordered slots and current vehicle membership; riders are omitted from chunk roots |
| `/data` adoption ignores incoming `Passengers` for every root | Strip the input list before adoption; preserve actual links and unrelated field adoption, including common data on loaded displays |
| Preserve duplicate-UUID semantics | Landed `PassengerSlot::Saved` still represents an in-tree duplicate skipped from level membership but retained for saving; existing #11807 regressions remain |
| Dismount, removal, tracking and execute relations agree | Ordered live children serve `/execute on`, LAN passenger publication, subtree removal and persistence; existing mob mount state and armor-stand vehicle state remain bridged |
| Unload/reload and player RootVehicle retain the graph | Generic root serialization and restoration, actual UUID attachment, omission of no-save entities and removal of the complete loaded tree |
| Level carriers retain or clear the same state | Existing EntityMemory level swaps retain ordered slots; transfer enumerates and removes every live member, then restores through the common loader |
| Non-mob riders follow their vehicles | Existing attachment/seat authority, typed entity position writers and dimensions cached by the existing entity-type registry read; normal seating does not issue teleports |

There is no second entity registry, persistent metadata format, per-query disk
read or fixture-specific target adapter. The existing prepared
`entity-types.json` read now also retains dimensions in memory. Player null
creation, the four `EntityTypes.Builder.noSave` registrations, and Marker's
boarding capability are pinned vanilla type policies. Per-kind entity codecs
remain the existing authority; this is not a replacement for every entity's
independent simulation or codec.

The official authority inspected was 26.3 `EntityType` creation and recursive
loading, `Entity.startRiding`, `Entity.load`, `Entity.saveWithoutId`,
`EntityTypes` registrations, `Marker`, `FishingHook`, and
`BlockAttachedEntity.setPos` / `HangingEntity.recalculateBoundingBox`.
The latter floors the hanging anchor and recalculates its bounds; it does not
retain an arbitrary floating-point anchor. Official bytecode and game files
remain private and are not committed.

## Evidence boundaries

The historical 177-record pair remains **125/177 before, 167/177 after**, with
all ten armor-stand diagnostics retained. Its 167-record gate does not prove
those ten records passed. The retained fixed runtime is `4adf2aff`, not the
published `d7c560ec` head. No binary equivalence between those heads is claimed.
The transfer archive was verified before exclusive extraction: 107946 bytes,
SHA256 `e3939ce9b4d26fba92f902b1349c4a2e42e4c32262bd8bd19b3ac5a2257545e2`,
70 safe regular members and 69 checked manifest payloads.

Fresh reference qualification preserves these separate results:

- V1 **587/623 RED**: 36 assertions incorrectly expected children to board a
  Marker root. The original program and RED log are unchanged.
- Prospective V2 **623/623 offline**, plus **57/57** independent ordered-save /
  bobber checks, from those same admitted raw replies. This was neither a Java
  replay nor an unchanged-V1 pass.
- The additional nested-tree, unload/reload, dismount/removal, duplicate and
  connected-player RootVehicle/save/restart contract passed **20/20**.
- The remaining leash-knot/lightning-bolt contract passed **22/22**: real live
  riders, actual vehicle links, omitted saves, refused children on no-save
  vehicles, and absence after save/restart. Already-admitted families were not
  replayed.
- The no-save transfer check retains a distinct **12/18 RED** V1: unrestricted
  `@e` was not limited to the selected level. Prospective V2 uses explicit
  distance bounds and passed **18/18** for actual source absence, destination
  identity/link, continued save omission and no-save roots. This is a fresh,
  corrected program, not an unchanged-V1 pass. Earlier admitted contracts were
  not replayed.

Raw typed bodies, replies, sentinels, clocks, callbacks, region/player-file
witnesses, helper versions and SHA manifests are retained privately. The
connected player's actual saved file had SHA256
`b197b0c2892decd936b238436d949424eb913843449d8e17ceff3283afb32c37`.

On frozen source `1453e0e614af4b2d04dd0d228a94162b5d459910`, the meaningful
loaded-Game baseline found **zero live armor-stand riders instead of one**.
Its sealed Mini executable is Mach-O arm64 (not ELF), SHA256
`9fcc1ba9ea7f113b75d48caf2e3c6ad99f1607e61a3a5c1d56529c29d4fda29f`.
The same meaningful assertions pass after the repair. A later meaningful
native regression also retained a RED for dropping a no-save rider during
level transfer. `vehicle_transfer_tag` now snapshots the live graph independently
of the disk-save filter, following `Entity.teleportCrossDimension`.
Seven completion tests cover the admitted families, ordered live/data/save/reload behavior, no-save
policies, cold-registry UUID preflight, seating, player-root restoration and
nested cross-dimension transfer, including live no-save riders. Original invalid-passenger and duplicate-UUID
controls remain separate tests.

The earlier final6 attempt was a compile-only checker error (`parse_snbt`
returns `(Tag, usize)`); its source and log are retained. It is not a gameplay
failure or a passing run. The subsequent release10 ownership-isolation attempt
also retained a compile-only privacy error: the new graph helper could not call
the command module's private helper. Moving the new helper alongside the
unchanged owned method corrected that boundary; release10 was not relabeled.

## Checks and retained REDs

The completed first `pr_ready --quick` run recorded Python **829 passed**,
client **1309 passed**, docs **0**, and Clippy **RED** for the obsolete
production `vehicle_tree` helper. Its wrapper exit was 0, child `pr_ready`
exit 1 and disk guard exit 0. That run was not restarted or relabeled. The
legacy pure helper is now test-only; subsequent workspace and server Clippy
checks passed. Final source checks and exact publication-head results are
reported in the PR.

A broader resources-enabled persistence run recorded **35 passed / 11 RED**.
Every RED reproduced on the sealed pristine executable:

- `h13076_loaded_anger_end_time_is_kept`
- `a_mob_saves_the_attribute_instances_it_has`
- `load_created_and_loaded_instances_stay_and_an_empty_list_is_saved`
- `loaded_camel_pose_timestamps_survive_actual_save_and_tick`
- `h262_loaded_noai_enderman_actual_ticks_and_model_projection`
- `a_sulfur_cube_reads_its_body_archetype_modifiers_after_a_tick`
- `command_knockback_changes_reach_the_cache_without_archetype_values`
- `explicit_knot_check_counts_only_running_entity_ticking_turns`
- `explicit_knot_removal_preserves_a_real_lead_created_knot`
- `summoned_knots_check_actual_fence_support_on_tick_101`
- `a_trader_llama_leaves_the_level_in_the_tick_its_delay_runs_out`

Those logs remain RED. The optional 557 affected-harness sweep is unrun.
No all-parity-green claim follows from the focused results.

All Rust build/test invocations use jobs 2, test threads 2, incremental 0 and local
`DEV/TEST/FAST_DEBUG=0` overrides, without changing optimization, assertions,
overflow checks or features. The unchanged entry/hard guards are 10/5 GiB for
Rust and 5 GiB for Java, with one Mini heavy job at a time. Source-specific
build receipts and executable hashes remain retained; no old/new executable
identity or performance equivalence is asserted.

## Ownership

The assigned issue remains #11822 / TheMrZZ. PR #14512's
`native_vehicle_of` / `native_passengers_of` methods in `native_travel.rs`
remain its owner's work; this change supplies an ordered graph independently.
PR #13845 owns the Guest arms of `ride_vehicle`; that method is unchanged
from main. The new `passenger_vehicle_target` graph helper delegates player /
Guest lookup to it. PR #14716's `remote_removal.rs`, and stand/dispenser owner
methods, are untouched. The landed #11831 partial and #12458 duplicate policy are preserved
as history rather than represented as a complete earlier fix.
