# EH3D-PROJECT-PASSPORT: requirements

This register is the single source of the state of each capability. Status values: planned, prototype, implemented, validated. Five requirements are `prototype` (code and tests exist, only on data made by the tests) and one is `planned` (PPAS-005); none has been used on real data or validated. High test coverage is not proof of geometric or electrical correctness, so each requirement says how it will be checked and what result is expected.

| Requirement | Description | Status | How it will be checked | Expected result | Known limits |
|---|---|---|---|---|---|
| PPAS-001 | Build a passport from a project and its history. | prototype | automatic: reference projects | The passport lists every required section. | The passport has fixed sections that are always written; the building of a passport from a project and its history is not written (`hashfiles` only gives the references of files). |
| PPAS-002 | Regenerating the report of an older revision gives the same result. | prototype | automatic | Identical passport content. | The canonical content (sections sorted) gives the same bytes and the same hash; the regeneration of a report needs the checks themselves, which are not here. |
| PPAS-003 | The passport allows the manufacturing files to be rebuilt. | prototype | automatic: rebuild and compare hashes | Rebuilt files match the recorded hashes. | `verify` finds the files of a folder that are missing or changed against the recorded hashes; it does not rebuild the files. |
| PPAS-004 | Record the content hash of exported packages. | prototype | automatic | The hash follows the container rule. | Records the SHA-256 of a package like that of any other file (64 lower-case hexadecimal digits); the container rule itself belongs to EH3D-COMMON. |
| PPAS-005 | Optional signature. | planned | automatic plus review | A signed passport verifies; a modified one does not. | Not started: no limits known yet. |
| PPAS-006 | State limitations and pending decisions explicitly. | prototype | automatic | Empty sections are written as empty, not omitted. | A missing section is an error when reading, and an empty one is written as empty; the text of a limitation or a pending decision is not interpreted. |

Rules: a claim of correctness names the corpus, the tolerance and the tool version it holds for. A phase is complete when the measured rate does not drop, every failure is classified and fixed or documented, and the measured figure is reported.
