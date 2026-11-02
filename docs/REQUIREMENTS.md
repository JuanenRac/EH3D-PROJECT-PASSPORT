# EH3D-PROJECT-PASSPORT: requirements

This register is the single source of the state of each capability. Status values: planned, prototype, implemented, validated. Every requirement is `planned` today: nothing is implemented or verified. High test coverage is not proof of geometric or electrical correctness, so each requirement says how it will be checked and what result is expected.

| Requirement | Description | Status | How it will be checked | Expected result | Known limits |
|---|---|---|---|---|---|
| PPAS-001 | Build a passport from a project and its history. | planned | automatic: reference projects | The passport lists every required section. | Not started: no limits known yet. |
| PPAS-002 | Regenerating the report of an older revision gives the same result. | planned | automatic | Identical passport content. | Not started: no limits known yet. |
| PPAS-003 | The passport allows the manufacturing files to be rebuilt. | planned | automatic: rebuild and compare hashes | Rebuilt files match the recorded hashes. | Not started: no limits known yet. |
| PPAS-004 | Record the content hash of exported packages. | planned | automatic | The hash follows the container rule. | Not started: no limits known yet. |
| PPAS-005 | Optional signature. | planned | automatic plus review | A signed passport verifies; a modified one does not. | Not started: no limits known yet. |
| PPAS-006 | State limitations and pending decisions explicitly. | planned | automatic | Empty sections are written as empty, not omitted. | Not started: no limits known yet. |

Rules: a claim of correctness names the corpus, the tolerance and the tool version it holds for. A phase is complete when the measured rate does not drop, every failure is classified and fixed or documented, and the measured figure is reported.
