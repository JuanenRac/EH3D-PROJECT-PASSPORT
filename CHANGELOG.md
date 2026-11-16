# Changelog

All notable changes to this project are documented here.

## [Unreleased]

- **A file that is a symbolic link is not hashed by default:** the new setting `reject_symlinks` (on by default) makes `hash_file`, `hashfiles` and `verify` refuse a file that is a symbolic link, because the link could point outside the folder of the project; `reject_symlinks = off` follows links as before. A test covers both (on systems that have links).
- **The hash of a file stops at the limit of bytes while it reads:** besides looking at the size first, the reading counts what it takes and refuses a file that grows, or a pipe or a device that says it is empty, once it passes `max_hashed_bytes`.

- **The count of Rust tests in the README and in the status page is the real one.**

- **`compare` also compares the checks, the limitations, the pending decisions, the project and the date:** two passports that differed only in a check (a verdict or the hash of its report), a limitation, a pending decision, the project name or the date compared as equal and the command exited with 0. They are now listed one line each, and a passport whose content differs in any other way is reported as `other content changed`. Tested for each section.

- **A simulated project folder (`sim`):** a host twin that makes the files of a folder from a seed, writes them, breaks one on purpose (missing, one bit flipped, cut, grown, or replaced by a folder) and writes it back, so `verify` can be tried on every failure and its recovery with no real project; the same seed always gives the same files, faults and findings. Six new tests cover each fault, a seeded campaign repeated 25 times, the limit of the size to hash and the unsafe path. It has only run on a computer: no real project folder, drive failure or file-system error has been seen.

## [0.0.3] - Shared validation tools, a split CI and a pinned Rust toolchain

- **A picture of the command line in the README:** `images/screens/project-passport-01-cli.png` shows `hashfiles`, `check` on a good passport and `verify` finding a file that changed, taken from the built program; it is shown in the eight READMEs.
- **The shared validation tools are updated:** the validator also runs the workflow and lessons checks of the suite with their latest rules, including a check to run before every push.
- **The CI validation checks the workflows and the lessons of past CI failures against a baseline:** `tools/workflow_hygiene.py` and `tools/lessons_lint.py` (copies of the shared tools) run in the validation, and only findings that are not in `tools/workflow_hygiene_baseline.json` and `tools/lessons_lint_baseline.json` fail it, so what exists today stays as it is and a new one is caught. `python tools/workflow_hygiene.py --root . --detail` and `python tools/lessons_lint.py --repo .` list the findings.
- **The validation of the CI is the reusable one of the documentation hub:** the job `validate` calls `validate.yml` of EH3D-DOCS (pinned by commit) and the check of the CHANGELOG is a job of its own.
- **The CI is split in two workflows:** the validation of the repository (manifest, documentation, pictures, tools) stays in `ci.yml` and the Rust jobs move to `rust.yml`, which does not run for a change that only touches Markdown files or pictures, so a change of the documentation no longer builds the code on two systems, and they keep a cache of the build. The names of the jobs are the same.
- **The Rust toolchain is pinned:** `rust-toolchain.toml` fixes the version of Rust (the same one in every repository of the suite) with `rustfmt` and `clippy`, the workflows install that version and not `stable`, so a new release of Rust cannot turn the CI red by itself.

## [0.0.2] - First release: the library, the command line, the stable error codes and the tests

- **Brand:** Add the application icon and the logos of the program.
- **Shared checks updated:** the validator also compares the badge and the Version row of the README with the manifest, fails when one of its checks cannot run, and the CHANGELOG step of the workflow reads the base branch from the environment.

- Add the library: the passport with its sections always written (tools and versions, input and output files with their SHA-256, results of the checks, limitations and pending decisions), the canonical order and hash, the comparison with the files of a folder and the comparison of two passports.
- Report, with stable codes `PHOENIX-PASS-001` to `PHOENIX-PASS-006`: bad hashes, unsafe paths, duplicates, tools without a version, files whose content changed and files that are missing.
- Add the command line (`check`, `verify`, `hash`, `compare`, `hashfiles`) and a settings file. Run format, lints and tests of the Rust code on Linux and Windows in the CI.
- **The workflows are hardened:** every action is pinned by its commit with its version in a comment, and the checkouts keep no credentials.

## [0.0.1] - The base of the repository

- **The base that every repository of the suite has:** the manifest, the README files in eight languages, this changelog, the CI workflow, the issue and pull-request templates, and the shared checks of EH3D-DOCS (the repository check and the drawers of the two README pictures).
- **A planned project, described honestly:** the scope, the requirements (all planned), the relations with other repositories, the threat model, the decisions that apply and the rules for writing code are in `docs/`; there is no code yet.
