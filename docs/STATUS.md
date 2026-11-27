# EH3D-PROJECT-PASSPORT: status

## Current state

EH3D-PROJECT-PASSPORT is early code on a computer: a Rust library and a command line (`check`, `verify`, `hash`, `compare`, `hashfiles`) that read a passport, check it, compare it with the files of a folder and compare two passports. A simulated project folder with seeded faults and its repair checks that the verification finds exactly what was broken. All of it has run only on data made by its own tests; the generator from a project, the rebuilding of the manufacturing files and the signature are not written, and no requirement is validated.

## Capabilities

The official state of each capability is in `REQUIREMENTS.md`. The register has 22 requirements: 16 implemented, 5 prototype, 1 planned and 0 validated. By progress: 16 done, 5 partial, 1 pending and 0 blocked. This page never states a state that `REQUIREMENTS.md` does not.

## Open questions

* Key management for the optional signature.
* Which sections the generator takes from each tool of the suite.

## What it needs from other repositories

* EH3D-COMMON: the content hash and canonical JSON.
* EH3D-DESIGN-HISTORY: the history it records.
* EH3D-DESIGN-VERIFIER: the results of the checks it records.

## Known technical debt and omitted tests

* 59 tests of the Rust code (nine of them keep this register true) and the 68 of the base (65 of them test the shared tools of the base); the Rust tests build their own passports and folders and never touch a real project.
* Seven of the tests run the simulation (`src/lib.rs`): the seeded folder, its faults and their repair, a campaign that must find exactly the files it broke, a file over the limit and an unsafe path.
* `cargo deny` is not run yet.
