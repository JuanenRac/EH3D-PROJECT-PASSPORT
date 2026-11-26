# EH3D-PROJECT-PASSPORT: threat model

Basic model for early code. The rows marked partial have a mitigation written and tests for it, run only on data made by the tests themselves; the rows marked planned have nothing yet. No row is assessed against real data.

| Threat | Planned mitigation | Planned test | State | Residual risk |
|---|---|---|---|---|
| A passport that claims more than was verified | every section is required, bad hashes, paths, duplicates and missing versions are flagged, and verify reports the files that changed or are missing | `every_section_is_required_and_unknown_fields_are_refused`, `bad_hashes_paths_duplicates_and_missing_versions_are_flagged`, `verify_finds_the_files_that_changed_or_are_missing_and_never_writes` | partial | only what the hashes of the listed files can show |
| A forged or modified passport | the files listed in the passport are hashed and verified one by one, and the same content in another order gives the same hash | `verify_finds_the_files_that_changed_or_are_missing_and_never_writes`, `the_same_content_in_another_order_has_the_same_hash_and_bytes` | partial | the passport itself is not signed: a passport replaced together with its files is not detected |
| Private data in a shared passport | explicit redacted export | leak test | planned | not assessed: nothing is implemented |
| Corrupt or malicious project or input files (oversized entries, path tricks, deep nesting, resource exhaustion) | an unsafe path is reported and never read, a symbolic link (the file, or a folder on the way to it) is refused unless the settings follow links, a reader that never ends stops at the limit, and big files and oversized input are refused | `an_unsafe_path_in_the_passport_is_reported_and_never_read`, `a_symbolic_link_is_refused_unless_the_settings_follow_links`, `a_reader_that_never_ends_stops_at_the_limit`, `hashing_files_refuses_unsafe_paths_and_big_files` | partial | hostile samples written by hand; no fuzzing yet |
| Temporary files, logs and reports that reveal private designs | no private data in outputs; redaction | automated pattern search | planned | not assessed: nothing is implemented |
| Compromised dependency or binary | pinned versions, licence and advisory checks, software bill of materials for releases | CI checks | planned | not assessed: nothing is implemented |
| Power loss or forced close during a save | atomic write: temporary file, then rename; old file never deleted first | interruption tests at controlled points | planned | not assessed: nothing is implemented |

Minimum negative tests once there is code: hostile files and resource limits; secrets absent from logs, errors and reports; pinned dependencies and licence review before publishing. Report a vulnerability privately as described in SECURITY.md, without attaching private designs.
