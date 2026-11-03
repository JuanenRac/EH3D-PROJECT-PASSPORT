# EH3D-PROJECT-PASSPORT: status

## Current state

EH3D-PROJECT-PASSPORT is early code. Version 0.0.1 holds only the base that every repository of the suite carries (manifest, README files in eight languages, changelog, CI, shared checks) and the written scope, requirements, dependencies, threat model and decisions. It also has a Rust library and a command line that read a passport, check it, compare it with the files of a folder and compare two passports. They have run only on data made by their own tests; nothing has been run on a machine with a real project, and no requirement is validated.

## Capabilities

The official state of each capability is in REQUIREMENTS.md: 6 requirements: 5 `prototype` and 1 `planned` (PPAS-005). This page never states a state that REQUIREMENTS.md does not.

## Open questions

* Key management for the optional signature.

## Needs from other repositories

* EH3D-COMMON: content hash and canonical JSON
* EH3D-DESIGN-HISTORY: the history it records

## Known technical debt and omitted tests

13 tests of the Rust code and the 3 of the base; the Rust tests build their own passports and folders and never touch a real project. `cargo deny` is not run yet.
