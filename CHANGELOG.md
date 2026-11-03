# Changelog

All notable changes to this project are documented here.

## [Unreleased]
- **Shared checks updated:** the validator also compares the badge and the Version row of the README with the manifest, fails when one of its checks cannot run, and the CHANGELOG step of the workflow reads the base branch from the environment.

- Add the library: the passport with its sections always written (tools and versions, input and output files with their SHA-256, results of the checks, limitations and pending decisions), the canonical order and hash, the comparison with the files of a folder and the comparison of two passports.
- Report, with stable codes `PHOENIX-PASS-001` to `PHOENIX-PASS-006`: bad hashes, unsafe paths, duplicates, tools without a version, files whose content changed and files that are missing.
- Add the command line (`check`, `verify`, `hash`, `compare`, `hashfiles`) and a settings file. Run format, lints and tests of the Rust code on Linux and Windows in the CI.
- **The workflows are hardened:** every action is pinned by its commit with its version in a comment, and the checkouts keep no credentials.

## [0.0.1] - The base of the repository

- **The base that every repository of the suite has:** the manifest, the README files in eight languages, this changelog, the CI workflow, the issue and pull-request templates, and the shared checks of EH3D-DOCS (the repository check and the drawers of the two README pictures).
- **A planned project, described honestly:** the scope, the requirements (all planned), the relations with other repositories, the threat model, the decisions that apply and the rules for writing code are in `docs/`; there is no code yet.
