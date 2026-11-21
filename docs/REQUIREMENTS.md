# EH3D-PROJECT-PASSPORT: requirements

This register is the single source of the state of each capability: `STATUS.md` and the README files cite it and never state a state that it does not. A state is one of `planned`, `prototype`, `implemented` and `validated`. `validated` means that a person has run the capability on a real machine and the evidence of the manifest says `bench` or `field`; nothing here is validated, because nothing of this program has been run on a real project or with real manufacturing files.

The register has 22 requirements: 16 implemented, 5 prototype, 1 planned and 0 validated. By progress: 16 done, 5 partial, 1 pending and 0 blocked. High coverage is not proof of correctness, so each row says how it is checked: the test that names it, and what it shows.

## How to read the register

* **ID:** `PPAS-NNN`, never reused and never renumbered; a new capability takes the next number.
* **State:** `implemented` is code that is complete and tested on a computer, `prototype` is part of the capability, `planned` is not written.
* **Progress:** `done`, `partial`, `pending` (can be written without hardware) or `blocked` (waits for hardware or for another program of the suite).
* **Evidence:** `simulator` when the test runs against a seeded simulated project folder whose files are broken on purpose, `host-test` when it checks a calculation, a reader or a screen-less piece, `none` when nothing is written yet.
* **Test:** the acceptance criterion: `file::name` is the test `name` of `tests/file.rs` (`src/file::name` is a test inside `src/file.rs`), which must exist. For a capability that is not written the cell says `planned:` and the name of the test that will show it; the day that test exists, the row must change.
* **Limits:** what is not done or not proven; a row that is `partial` or `blocked` always says it.

The check is `tests/reqs.rs`: it reads this table, finds each test and fails when a state, a test or a count does not agree.

## The passport

| ID | Requirement | State | Progress | Evidence | Test | Limits |
|---|---|---|---|---|---|---|
| PPAS-001 | Build a passport from a project and its history. | prototype | partial | host-test | `src/passport::a_clean_passport_has_no_findings` | The passport has fixed sections that are always written and can be checked; building one from a project and its history is not written (`hashfiles` only gives the references of files). |
| PPAS-002 | Regenerating the report of an older revision gives the same result. | prototype | partial | host-test | `src/passport::the_same_content_in_another_order_has_the_same_hash_and_bytes` | The canonical content (sections sorted) gives the same bytes and the same hash; the regeneration of a report needs the checks themselves, which are not here. |
| PPAS-003 | The passport allows the manufacturing files to be rebuilt. | prototype | partial | host-test | `src/passport::verify_finds_the_files_that_changed_or_are_missing_and_never_writes` | `verify` finds the files of a folder that are missing or changed against the recorded hashes; it does not rebuild the files. |
| PPAS-004 | Record the content hash of exported packages. | prototype | partial | host-test | `src/cli::hashfiles_then_check_verify_hash_and_compare` | A package is hashed like any other file (SHA-256 in 64 lower-case hexadecimal digits); the container rule itself belongs to EH3D-COMMON. |
| PPAS-005 | Optional signature: a signed passport verifies and a modified one does not. | planned | pending | none | `planned:signature::a_signed_passport_verifies_and_a_modified_one_does_not` | Not written: it needs a key policy that nobody has decided. |
| PPAS-006 | State the limitations and the pending decisions explicitly. | prototype | partial | host-test | `src/passport::every_section_is_required_and_unknown_fields_are_refused` | A missing section is an error when reading and an empty one is written as empty; the text of a limitation or a pending decision is not interpreted. |
| PPAS-007 | The limitations, the pending decisions, the project and the date are compared too. | implemented | done | host-test | `src/passport::checks_limitations_pending_project_and_date_are_compared_too` | - |
| PPAS-008 | The hash of a fixed passport is a known value, so a change of the format cannot go unnoticed. | implemented | done | host-test | `src/passport::the_hash_of_a_fixed_passport_is_the_known_one` | - |
| PPAS-009 | Bad hashes, unsafe paths, duplicates and missing tool versions are findings. | implemented | done | host-test | `src/passport::bad_hashes_paths_duplicates_and_missing_versions_are_flagged` | - |
| PPAS-010 | Two passports are compared line by line in a fixed order. | implemented | done | host-test | `src/passport::two_passports_are_compared_line_by_line_in_a_fixed_order` | - |

## The files and the limits

| ID | Requirement | State | Progress | Evidence | Test | Limits |
|---|---|---|---|---|---|---|
| PPAS-011 | A symbolic link is refused unless the settings follow links. | implemented | done | host-test | `src/passport::a_symbolic_link_is_refused_unless_the_settings_follow_links` | - |
| PPAS-012 | Hashing files refuses unsafe paths and big files. | implemented | done | host-test | `src/passport::hashing_files_refuses_unsafe_paths_and_big_files` | - |
| PPAS-013 | A reader that never ends stops at the limit. | implemented | done | host-test | `src/passport::a_reader_that_never_ends_stops_at_the_limit` | - |
| PPAS-014 | The size limits of the passport apply. | implemented | done | host-test | `src/passport::the_size_limits_apply` | - |

## The command line and the settings

| ID | Requirement | State | Progress | Evidence | Test | Limits |
|---|---|---|---|---|---|---|
| PPAS-015 | Bad input and bad arguments exit with 2. | implemented | done | host-test | `src/cli::bad_input_and_bad_arguments_exit_2` | - |
| PPAS-016 | A settings file changes the rule about tool versions. | implemented | done | host-test | `src/cli::a_settings_file_changes_the_rule_about_tool_versions` | - |
| PPAS-017 | The defaults are conservative and a bad line of the settings is refused. | implemented | done | host-test | `src/config::lines_change_values_and_bad_lines_are_refused` | - |

## Simulation of a project folder

| ID | Requirement | State | Progress | Evidence | Test | Limits |
|---|---|---|---|---|---|---|
| PPAS-018 | A seeded folder is the same for the same seed, and a folder just written verifies clean. | implemented | done | simulator | `src/lib::the_same_seed_gives_the_same_folder_and_another_seed_a_different_one` | The folder is made by the twin, not exported by a real project. |
| PPAS-019 | Each fault gives its own finding and the repair brings the folder back. | implemented | done | simulator | `src/lib::each_fault_gives_its_own_finding_and_the_repair_brings_the_folder_back` | - |
| PPAS-020 | A seeded campaign finds exactly the files it broke and repeats exactly. | implemented | done | simulator | `src/lib::a_seeded_campaign_finds_exactly_the_files_it_broke_and_repeats_exactly` | - |
| PPAS-021 | A file over the limit stops verify with an error and the default limit recovers. | implemented | done | simulator | `src/lib::a_file_over_the_limit_stops_verify_with_an_error_and_the_default_limit_recovers` | - |
| PPAS-022 | An unsafe path in the passport is reported and never read. | implemented | done | simulator | `src/lib::an_unsafe_path_in_the_passport_is_reported_and_never_read` | - |
