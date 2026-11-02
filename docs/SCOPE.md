# EH3D-PROJECT-PASSPORT: scope

Status: **planned**. EH3D-PROJECT-PASSPORT has no code. This page says what it is meant to do, what it will not do, and where it sits. Nothing here has been built or run.

## Purpose

EH3D-PROJECT-PASSPORT is planned as a generator of a storable, reproducible technical passport to audit a whole project.

Generator of the technical passport of a project: everything needed to reproduce and review it.

## What it is planned to do

* Collect in one record: versions of the files and of the data model, components and libraries, rules and profiles, versions of the simulation engines, results of the checks, manufacturing files (by hash), laboratory tests, known limitations and pending decisions.
* Be reproducible: the passport allows the manufacturing files to be rebuilt, and regenerating the report of an old revision gives the same result.
* Record the content hash of exported packages.
* Optionally carry a signature, never required to open a local project.
* Planned as a library with a command-line interface and a server task kind.

## What it will not do

* It is not a certificate and makes no claim of compliance with any standard.
* No signing key is ever stored in a repository.
* It does not run checks itself: it records the results of the other tools and their versions.

## Planned shape

A library without a window, written in Rust (planned), with a command-line interface and, where it applies, a kind of task for the server. Programs of the suite exchange work through project files or through the server, never by depending on each other.

Crates, names and file layout will be decided together with the first code; no crate exists yet.

## Where it sits

It belongs to the PHOENIX family of the EH3D Fabrication Suite: independent repositories that relate through versioned contracts. The design tool (EH3D-PHOENIX-PCB) does not depend at build time on the repositories that depend on it.
See DEPENDENCIES.md for the relations, REQUIREMENTS.md for the requirements and STATUS.md for the honest state.
