# EH3D-PROJECT-PASSPORT: scope

Status: **early code**. EH3D-PROJECT-PASSPORT is a library and a command line that check a passport, compare it with the files of a folder and compare two passports. It has been tried on data made by its own tests only. This page says what it is for, what it will not do and where it sits among the other programs of the suite.

## Purpose

The technical passport of a project: one record that holds what is needed to review a project and to reproduce it (the versions of the files, the components, the rules and profiles, the results of the checks, the manufacturing files by hash, the laboratory tests, the known limitations and the pending decisions), stored in a canonical form so that the same content always gives the same bytes and the same hash.

## What it does

* Reads a passport (a JSON file within a size limit) in which every section is required and unknown fields are refused, and checks it: bad hashes, unsafe paths, duplicated entries and missing tool versions are findings.
* Hashes the files of a folder with SHA-256 (`hashfiles`), refusing unsafe paths, links (unless the settings say to follow them) and files over the size limit.
* Verifies the files of a folder against the hashes of a passport: it finds the files that changed or are missing, and never writes.
* Compares two passports line by line in a fixed order, including the limitations, the pending decisions, the project and the date.
* Writes the passport in a canonical form: the same content in another order has the same hash and the same bytes.
* Runs a simulated project folder with seeded faults (a file missing, flipped, cut, grown or replaced by a folder) and its repair.

## What it will not do

* It is not a certificate and makes no claim of compliance with any standard.
* No signing key is ever stored in a repository, and a signature is never required to open a local project.
* It does not run checks itself: it records the results of the other tools and their versions.
* It does not build a passport from a project yet, and it does not rebuild the manufacturing files from one.
* It does not claim that anything was proven on a real project: the evidence of every capability is `simulator` or `host-test`.

## Where it sits

* **It reads:** a passport file (JSON), the files of a folder it is asked to hash or to verify, and a settings file (the rule about tool versions, whether links are followed, the size limits).
* **It writes:** the passport or the hashes it is asked to print, to the standard output or to a file the person names; it never writes to the files it verifies.
* **It is read by:** nothing reads it yet: the server and the clients will store and show the passport once the generator exists.
* **It depends on:** nothing of the suite at build time; the history it records comes from EH3D-DESIGN-HISTORY and the content hash from EH3D-COMMON.

## What it never decides

Whether a project is fit to be made, and the limits, the interlocks and the emergency stop of any machine, belong to the person who signs it and to the machine. A passport that verifies shows that the files are the ones that were recorded; it does not show that they are right. Nothing here starts motion.

The requirements of the program, with the state of each and the test that shows it, are in `REQUIREMENTS.md`; the state in a paragraph is in `STATUS.md`.
