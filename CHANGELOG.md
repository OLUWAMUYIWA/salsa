# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.20.0](https://github.com/OLUWAMUYIWA/salsa/compare/salsa-v0.19.0...salsa-v0.20.0) - 2025-03-31

### Added

- Drop `Debug` requirements and flip implementation defaults ([#756](https://github.com/OLUWAMUYIWA/salsa/pull/756))

### Fixed

- Use `changed_at` revision when updating fields ([#778](https://github.com/OLUWAMUYIWA/salsa/pull/778))

### Other

- Normalize imports style ([#779](https://github.com/OLUWAMUYIWA/salsa/pull/779))
- Clean up `par_map` a bit ([#742](https://github.com/OLUWAMUYIWA/salsa/pull/742))
- Fix typo in comment ([#777](https://github.com/OLUWAMUYIWA/salsa/pull/777))
- Document most safety blocks ([#776](https://github.com/OLUWAMUYIWA/salsa/pull/776))
- Use html directory for mdbook artifact ([#774](https://github.com/OLUWAMUYIWA/salsa/pull/774))
- Move `verified_final` from `Memo` into `QueryRevisions` ([#769](https://github.com/OLUWAMUYIWA/salsa/pull/769))
- Use `ThinVec` for `MemoTable`, halving its size ([#770](https://github.com/OLUWAMUYIWA/salsa/pull/770))
- Remove unnecessary query stack acess in `block_on` ([#771](https://github.com/OLUWAMUYIWA/salsa/pull/771))
- Replace memo queue with append-only vector ([#767](https://github.com/OLUWAMUYIWA/salsa/pull/767))
- update boxcar ([#696](https://github.com/OLUWAMUYIWA/salsa/pull/696))
- Remove extra page indirection in `Table` ([#710](https://github.com/OLUWAMUYIWA/salsa/pull/710))
- update release steps ([#705](https://github.com/OLUWAMUYIWA/salsa/pull/705))
- Remove some unnecessary panicking paths in cycle execution ([#765](https://github.com/OLUWAMUYIWA/salsa/pull/765))
- *(perf)* Pool `ActiveQuerys` in the query stack ([#629](https://github.com/OLUWAMUYIWA/salsa/pull/629))
- Resolve unwind safety fixme ([#761](https://github.com/OLUWAMUYIWA/salsa/pull/761))
- Enable Garbage Collection for Interned Values ([#602](https://github.com/OLUWAMUYIWA/salsa/pull/602))
- bug [salsa-macros]: Improve debug name of tracked methods ([#755](https://github.com/OLUWAMUYIWA/salsa/pull/755))
- Remove dead code ([#764](https://github.com/OLUWAMUYIWA/salsa/pull/764))
- Reduce unnecessary conditional work in `deep_verify_memo` ([#759](https://github.com/OLUWAMUYIWA/salsa/pull/759))
- Use a `Vec` for `CycleHeads` ([#760](https://github.com/OLUWAMUYIWA/salsa/pull/760))
- Use nextest for miri test runs ([#758](https://github.com/OLUWAMUYIWA/salsa/pull/758))
- Pin `half` version to prevent CI failure ([#757](https://github.com/OLUWAMUYIWA/salsa/pull/757))
- rewrite cycle handling to support fixed-point iteration ([#603](https://github.com/OLUWAMUYIWA/salsa/pull/603))

## [0.19.0](https://github.com/salsa-rs/salsa/compare/salsa-v0.18.0...salsa-v0.19.0) - 2025-03-10

### Fixed

- fix typo
- fix enums bug

### Other

- Have salsa not depend on salsa-macros ([#750](https://github.com/salsa-rs/salsa/pull/750))
- Group versions of packages together for releases ([#751](https://github.com/salsa-rs/salsa/pull/751))
- use `portable-atomic` in `IngredientCache` to compile on `powerpc-unknown-linux-gnu` ([#749](https://github.com/salsa-rs/salsa/pull/749))
- Store view downcaster in function ingredients directly ([#720](https://github.com/salsa-rs/salsa/pull/720))
- Some small perf things ([#744](https://github.com/salsa-rs/salsa/pull/744))
- :replace instead of std::mem::replace ([#746](https://github.com/salsa-rs/salsa/pull/746))
- Cleanup `Cargo.toml`s ([#745](https://github.com/salsa-rs/salsa/pull/745))
- Drop clone requirement for accumulated values
- implement `Update` trait for `IndexMap`, and `IndexSet`
- more correct bounds on `Send` and `Sync` implementation `DeletedEntries`
- replace `arc-swap` with manual `AtomicPtr`
- Remove unnecessary `current_revision` call from `setup_interned_struct`
- Merge pull request #731 from Veykril/veykril/push-nzkwqzxxkxou
- Remove some dynamically dispatched `Database::event` calls
- Lazy fetching
- Add small supertype input benchmark
- Replace a `DashMap` with `RwLock` as writing is rare for it
- address review comments
- Skip memo ingredient index mapping for non enum tracked functions
- Trade off a bit of memory for more speed in `MemoIngredientIndices`
- Introduce Salsa enums
- Cancel duplicate test workflow runs
- implement `Update` trait for `hashbrown::HashMap`
- Move `unwind_if_revision_cancelled` from `ZalsaLocal` to `Zalsa`
- Don't clone strings in benchmarks
- Merge pull request #714 from Veykril/veykril/push-synxntlkqqsq
- Merge pull request #711 from Veykril/veykril/push-stmmwmtprovt
- Merge pull request #715 from Veykril/veykril/push-plwpsqknwulq
- Enforce `unsafe_op_in_unsafe_fn`
- Remove some `ZalsaDatabase::zalsa` calls
- Remove outdated FIXME
- Replace `IngredientCache` lock with atomic primitive
- Reduce method delegation duplication
- Automatically clear the cancellation flag when cancellation completes
- Allow trigger LRU eviction without increasing the current revision
- Simplify `Ingredient::reset_for_new_revision` setup
- Require mut Zalsa access for setting the lru limit
- Split off revision bumping from `zalsa_mut` access
- Update `hashbrown` (0.15) and `hashlink` (0.10)
