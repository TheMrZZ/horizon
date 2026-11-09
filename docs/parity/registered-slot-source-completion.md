# Registered slot-source completion

This continues #13091 after the partial condition-read repair in PR #13107.
The original review, partial write failures and its condition-read evidence remain
historical evidence; the earlier partial description is not a completion claim.

A registered source now uses the same loaded `DataStore.slot_sources` snapshot
for condition reads, `/item modify`, replace/fill/override targets and `from`
sources. Nested holders resolve before consumption, preserving ordered group
terms and transform nesting. Holder-set tags use the existing pack load pass and
snapshot, with required members, nested tags, replacement and first-seen order.
Candidate source codecs and references are checked before a reload publishes a
new snapshot. A rejected candidate leaves the old runtime definitions available.
No query reads disk and no second registry is introduced.

The addressed provider and command executor are distinct. The implicit source is
`container`; `this` refers only to the actual command executor. Other legal
context parameters absent from the command context select nothing. An absent
executor never becomes the local or an online player. The addressed block must
be a valid container even when an explicit source selects a different identity.

The existing Slot adapters remain responsible for real players and guests,
mob/stand equipment, frames, carts and component parent writeback. H247/H308
collection consumption remains authoritative: live filter visitation, shared
selection limits, selected empty slots and replace/fill/override distribution
are not replaced by a flattened stack list. Dispenser equipment selection and
stand locks owned by #12937 are unchanged; #8277 equipment admissibility is a
separate limitation, not a hidden source-resolution fallback.

## Reviewable regression

`tools/fixtures/registered_slot_sources` contains the authored pack and declared
stage-A program. The actual Game function
`registered_sources_reach_live_read_write_from_and_executor_context_v2` loads the
pack, verifies enabled identity and registry publication, waits for actual loaded
chunks and exercises real command parsing, callbacks and physical slots. It
checks exact initial stacks, recipient mutations, executor identity, unless and
missing-ID controls, the full unchanged source chest and clocks.

Build a gameplay test executable under the caller's existing resource guards,
then run it in isolated homes and a scratch directory using:

```sh
python3 tools/test_registered_slot_sources.py \
  --test-binary /absolute/path/to/gameplay-test-executable \
  --resources /absolute/path/to/prepared/resources
```

The driver performs no build or Java replay. Its receipt records the executable
and fixture hashes, exit status and retained scratch log. It enforces a 5 GiB
engine guard; it does not establish source/build eligibility for an arbitrary
supplied executable.

The staged current qualification separately records block reads/mutations/from,
holders/tags and successful/failed reloads, unlocked entity adapters, a real
connected player with inventory/ender storage and no console fallback, all three
component collection families, direct inline holder/tag syntax, and invalid
addressed-provider controls. These are separate admissions and captures, not a
single aggregate run. Full raw replies, typed bodies, clocks and invalid attempts
remain available in the task evidence. Final measured results are reported with
the publication; this document alone does not assert that required checks pass.

## Checker boundary

The original V1 Game function and its RED result are retained. Its final
`gold_ingot[count=6]` predicate is valid Java: the pinned item argument registers
`count` as both a pseudo-component (`=`) and a subpredicate (`~`). The native
matcher currently handles pseudo-count only under `~`. That unrelated matcher
is unchanged by this repair. The six A and four F `count=` assertion failures
remain recorded; they are not relabeled as passes.

Prospective V2 changes that final assertion to the existing independent entity
NBT path, requiring exact `equipment.mainhand` item id and integer count 6. The
same command sequence, callbacks, physical block slots and unchanged source
checks remain. Labeled `count=`, `count~` and plain predicate diagnostics are
printed separately. V2 is a corrected assertion, not an unchanged-function pass.
Independent typed saved-body checks on the staged native captures verify the
actual mutations and component/recipient state without relying on `count=`.
