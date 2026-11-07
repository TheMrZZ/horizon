# Item command removal patches

The item argument parser accepts explicit component removals, but `command_components` discarded them before decoding the actual stack. The shared bridge now carries additions and removals through the existing typed `component_patch::Patch`, emitting the existing `!minecraft:*:{}` saved removal markers. The decoder, StackData, saved encoder, prototypes, validation and effective consumers are unchanged.

Pinned Java 26.3 `ItemParser.State.readComponents` calls `visitRemovedComponent`; the parser's visitor forwards that operation to `DataComponentPatch.Builder.remove`. The repair follows that boundary rather than treating a removal as an ordinary absent addition.

## Frozen command proof

Base: `b0a8cbf6fb08669e278ff584f14bcc9c1aba6a1c`. Issue: [#11592](https://github.com/horizon-rust/horizon/issues/11592).

The predeclared 211-record contract uses an actual loaded chest, registered item-replace commands, independent slot identity/count reads, saved marker presence and empty-compound type/count reads, complete custom_data, fresh result/success sentinels, an untouched second slot, actual player identity/mode/position and strict zero relative game time. No component predicates substitute for the saved stack reads.

Cases are plain/custom/nondefault iron helmets; short and namespaced max_damage removals; max_damage removal with max_stack_size 2/count 2; plain stone and max_stack_size removal; and ordinary helmet count rejection. The two item prototypes differ. The valid removed/count-2 command remains an admitted positive even though the independent H086 combination validator is absent from this source base.

The unchanged program SHA256 is `089c5d72aa7d06c73dbc2ab9bb56784d3bb68ad716cc15737a43389d99cdf4ca`. Fresh Java reference SHA256 is `df1b1658ca7125c80434c5acac992dab6a0f09493a2149b76db3c72950a8bace`. Pristine actual Game: **195/211**. All sixteen differences are removal presence/value/success/type observations in four cases. Every setup/control record passes. Fixed actual Game: **211/211**, using the identical script and fresh Java reference bytes.

```sh
python3 tools/test_item_command_removal_parity.py --reference-only --work-dir .local/removal-java
python3 tools/test_item_command_removal_parity.py --binary /path/to/frozen-game \
  --reference .local/removal-java/vanilla.jsonl --work-dir .local/removal-game
```

The actual registered Game regression first verifies the loaded helmet prototype, positive plain/nondefault writes and independent initial slots, then asserts the actual decoded removal model and real saved stack. On the untouched bridge it fails with `None` versus `Some(Compound({}))`. A resource-free command→decoded-stack→saved-stack regression fails at the same model assertion; it also covers fresh no-raw modeled removal saving, additions, namespace preservation and existing syntax errors. Both tests are identical before/after; the actual registered Game test passes fixed.

## Complete diagnostic contract

The separately predeclared `--full-diagnostic` program has 300 records. It includes every complete saved stack/components record, both absolute clocks, and three H086 invalid-combination commands with Java's exact rejection/unchanged destination/store expectations. Those cases are helmet sizes 2 and 99, and stone max_damage 10 with size 2. They are not converted into native-acceptance expectations. Every record is compared strictly.

```sh
python3 tools/test_item_command_removal_parity.py --full-diagnostic \
  --binary /path/to/frozen-game --work-dir .local/removal-full
```

The first Java full reference had absolute game time 305; the old full capture's Java had 301. A requested cross-session full-byte reference check was rejected, and its complete already-finished capture was retained. Offline completion of the same declared strict comparison validates all Java guards/callbacks and retains every native row: **243/300**. Only the two absolute-clock records differ between the two Java sessions; all other 298 Java records match. Fixed full diagnostic: **267/300**. All 33 remaining rows belong to the three H086 rejection/state/store diagnostics (31) or the two absolute-clock records. All 24 removal-related diagnostic differences are resolved. This does not establish raw-clock equality or full diagnostic parity. The canonical 211 reference remains byte-identical, with absolute values preserved in raw traces and relative delta zero strict. Full captures retain their own actual Java/native clock values without normalization.

All actual original/fixed residuals are published in [item-command-removal-residuals.jsonl](item-command-removal-residuals.jsonl). Complete scripts, declarations, references, outputs, trace logs, binaries, rejected reference assertion and build/test logs are retained under `.local/item-command-removal/`.

## Source reach and limits

The shared helper also materializes native browser and `mod_give` stacks, plus examples and existing item-sync/tooltip tests. Their ordinary-addition controls are affected checks; runtime proof here is the measured Game command route. Saved markers do not prove browser/mod/network or universal effective-gameplay parity.

PR [#11498](https://github.com/horizon-rust/horizon/pull/11498) independently validates effective damageable/stackable command combinations. Its count-first, validation-only view must remain independent of this materialization repair. When combined, retain both validation and removal preservation; its now-stale comment about materialization discarding removals can be updated then. No pending H086 production is imported. Transmute result-patch composition (#9819), generic removal effects and unrelated full-stack presentation remain separate.

## Local checks and review

`pr_ready.py --quick --base b0a8cbf6fb08669e278ff584f14bcc9c1aba6a1c` ran once on the final production patch: 1,203 affected Rust tests, 27 content-root tests, doc tests, rustfmt, Clippy and public-tree checks pass. The resource-free removal unit is among those Rust tests. The actual registered Game regression passed separately. Existing caller controls pass: 13 item-sync and 20 tooltip tests, with one existing tooltip test ignored.

Overall pr_ready exits 1: the Python suite reports 718 run, three failing optional-release subcases and one skip. Masks 0/1 fail with `musl[@]: unbound variable`; mask 2 fails with `arm64[@]: unbound variable`. These are the previously observed independent release-script failures, not a bridge repair. The 409 broader parity harnesses listed by `--quick` were not run.

Later main `7775a14526c03c354b16338c6d588e375030ef41` was reviewed read-only: no delta in the bridge/patch/parser/materialization callers, repository instructions or command fixture driver. Proof stays based on b0a8; no unrelated source update or pending patch was imported.
