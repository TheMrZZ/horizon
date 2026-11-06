# Generator sea level in guest spawn info

Issue #11436. Frozen actual main `9d11d0c710705ee0901c2186d78641048a6660b0`.

Two independent builtin worlds use the existing H330 real connected-client route: login, disconnect/save/rejoin, accepted death, ClientCommand action0, actual respawn and save. Own fresh Java26.3 validates each 75-record contract, including actual UUID/type/Health/mode/pose, loaded floor/air, real relative clocks and world/personal point separation. No fabricated packet or clock is used.

The full decoding proxy retains every emitted packet byte, count and order. `SpawnInfoDecode` uses the pinned official Login/Respawn codecs, rejects trailing bytes, and records every CommonPlayerSpawnInfo field. Both logins and the respawn resolve actual builtin Overworld dimension/type keys. Saved `world_gen_settings.dat` is retained and independently checked for seed12345 and actual flat/noise generator inputs in original/generated/Java/native worlds. The Game unit constructs real generators from the prepared bundle; no dimension-name table or fake prototype answer supplies sea level.

## Untouched proof

| Scene | Untouched exact records | Residuals |
| --- | --- | --- |
| Flat Overworld | 72/75 | Both logins and respawn: Java −63, native63 |
| Normal Overworld | 75/75 | None |

All other decoded SpawnInfo fields are retained exactly, including obfuscated seed, flags, previous mode, last-death location and portal cooldown. Raw packets, player bodies, absolute clocks and unrelated payload fields stay diagnostic; this is not complete packet-family parity.

## Implementation and context ordering

`VanillaTerrain::sea_level` exposes the existing loaded generator's accessor. The existing TickCtx gains only that value; the three Hub spawn-info callers consume it. Five actual Game context constructors read the currently active generator, without caching a dimension-based value. Eleven synthetic/integration contexts retain their previous63 default.

The fifth Game site is the existing `server_ctx` helper used by both level-change callers. Each caller already performs `enter_level(destination)` before building that context and sending `change_level`. The login path similarly enters the joining player's level before its context; cross-level guest respawn enters the target before its context. None of that ordering changes. Source reach includes those paths, but runtime coverage here is same-level builtin flat/normal login and death-respawn. Nether/End and portal transfers are not captured.

The reachable Game regression shares the existing H330 setup/assertions, preserving world/default-point versus personal landing. A new case installs real loaded flat → normal → flat generators in the same Game and checks actual emitted respawn sea levels, unchanged UUID/Health/landing/default-point order and clock. Untouched code fails the intended flat assertion (63 versus −63), after the real setup and respawn. This tests active generator changes, not a portal lifecycle.

If the generator is unavailable, the prior63 fallback remains. This is not custom-dimension, debug-world or unavailable-generator correctness. Terrain policy, protocol defaults, all other SpawnInfo fields and H330 spawn position remain unchanged.

## Evidence and reproduction

`tools/test_spawn_info_sea_level_parity.py` reuses the existing network route/client/proxy; `mods/horizon/src/test/java/dev/horizonlab/parity/SpawnInfoDecode.java` adds only bounded offline official-codec decoding of captured packets. The public wrapper relocates that decoder path and wraps main; its command program is the unchanged original capture program. Private `.local/h258/script-v1.py`, its predeclared hashes and both Java references remain byte-identical for old/fixed captures.

Private `.local/h258/` preserves full captures, saved worlds/player files, raw expected/actual reports, original source/test and immutable binaries, ownership searches, official source queries, generator-input audits and manifests. The untouched runtime is our own pristine9d binary retained before H098 edits, with source/hash provenance; it contains no H098 production. Unknown phase times remain null. No worktrees, Root payloads or pending production are used.

## Fixed pair

The original script and references remain unchanged: flat **72/75 → 75/75**, normal **75/75 → 75/75**. The exact original flat residuals are `spawn/0/sea_level`, `spawn/1/sea_level`, `spawn/2/sea_level`, each expected−63/actual63. Fixed residual lists are empty. Both captures also retain all ten decoded SpawnInfo fields; every fixed decoded value matches Java (three packets per scene), including the diagnostic seed/previous-mode/last-death values. Whole wire streams and full player bodies are retained without a whole-stream equality claim.

Java reference SHA256: flat `bc217db761af011bca82728a9f1508e1b38cca496d66a12aeb6d4aa61fd51a9b`; normal `cb8988975a3ab2af96865bc153e335f05c879c2115b82820722c8f66375e6470`.

## Local checks

The same H258 regression fails untouched at the intended sea-level assertion and passes fixed with identical unit features/profile. The original H330 control and new transition case both pass explicitly (2/2). Required checks passed 3,233 Rust tests, formatting, workspace/server Clippy and public-tree checks. Python ran712 with three existing failures in optional release archive combinations (`musl[@]`/`arm64[@]`: Bash unbound variables), one skipped; the overall readiness command exits1. The408 affected parity harnesses were listed for CI by `--quick`, not run locally.

Later main `13f70ad48` was source-reviewed: no generator, Hub or level-context delta. These runtime proofs/checks remain qualified to9d, without a latest-base runtime claim. An initial fixed-unit artifact selection was rejected by an identical-hash guard before execution; a same-command default-feature rebuild supplies the final passing pair. The rejected selection and all artifacts/logs remain private diagnostics, not an H258 failure.
