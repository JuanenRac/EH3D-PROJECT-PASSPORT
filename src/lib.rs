// SPDX-License-Identifier: GPL-3.0-or-later
//! EH3D-PROJECT-PASSPORT: the technical passport of a project, stored as a file and checkable against the files it names.
//!
//! A passport lists, in sections that are always written (an empty section is written as empty, never left out), the tools and their versions, the input files
//! of the design, the output files (manufacturing files and packages) with their SHA-256, the results of the checks, the known limitations and the pending
//! decisions. It reads no clock: a date is text the person supplies. It is canonical, so the same content gives the same hash, and `verify` reads the files
//! of a folder (never writing to them) and says which ones are missing or are no longer what the passport recorded.
//!
//! The optional signature of a passport is not written yet.
#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod cli;
pub mod config;
pub mod error;
pub mod passport;

pub use error::PassportError;
pub use passport::{FileRef, Passport};

/// The stable code of a finding. Codes are never reused.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Code {
    /// A hash that is not 64 lower-case hexadecimal digits.
    BadHash,
    /// A path that is empty, absolute, has `..`, a backslash or a drive.
    UnsafePath,
    /// The same path or tool listed twice.
    Duplicate,
    /// A tool without a version.
    MissingToolVersion,
    /// A file whose content is no longer what the passport recorded.
    FileMismatch,
    /// A file the passport names that is not in the folder.
    MissingFile,
}

impl Code {
    /// The text of the code, such as `PHOENIX-PASS-001`.
    pub fn as_str(self) -> &'static str {
        match self {
            Code::BadHash => "PHOENIX-PASS-001",
            Code::UnsafePath => "PHOENIX-PASS-002",
            Code::Duplicate => "PHOENIX-PASS-003",
            Code::MissingToolVersion => "PHOENIX-PASS-004",
            Code::FileMismatch => "PHOENIX-PASS-005",
            Code::MissingFile => "PHOENIX-PASS-006",
        }
    }
}

/// The simulation of a project folder: a host twin whose files are made from a seed, faults injected on purpose (a file gone, flipped, cut, grown or
/// replaced by a folder) and the repair that brings the folder back. It lets `verify` be tried on every failure with no real project, and the same
/// seed always gives the same files, the same faults and the same findings.
pub mod sim {
    use crate::passport::{FileRef, Passport, Tool};
    use sha2::{Digest, Sha256};
    use std::collections::BTreeMap;
    use std::fs;
    use std::io;
    use std::path::Path;

    /// A small deterministic generator (xorshift64*). It reads no clock and no operating-system source.
    #[derive(Debug, Clone)]
    pub struct SimRng(u64);

    impl SimRng {
        /// A generator for the seed (a zero seed is replaced, because xorshift cannot start from zero).
        pub fn new(seed: u64) -> SimRng {
            SimRng(if seed == 0 {
                0x9E37_79B9_7F4A_7C15
            } else {
                seed
            })
        }

        /// The next 64 bits.
        pub fn next_u64(&mut self) -> u64 {
            let mut x = self.0;
            x ^= x >> 12;
            x ^= x << 25;
            x ^= x >> 27;
            self.0 = x;
            x.wrapping_mul(0x2545_F491_4F6C_DD1D)
        }

        /// A number below `n` (`n` must be above zero).
        pub fn below(&mut self, n: usize) -> usize {
            (self.next_u64() % n as u64) as usize
        }
    }

    /// What goes wrong with one file of the folder.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Fault {
        /// The file is deleted.
        Missing,
        /// One bit of the content is flipped.
        FlipBit,
        /// The content is cut to a part of its size.
        Truncate,
        /// Bytes are appended.
        Grow,
        /// The file is replaced by a folder of the same name (it cannot be read as a file).
        Unreadable,
    }

    /// Every fault, in a fixed order, for the scenarios that go through all of them.
    pub const ALL_FAULTS: [Fault; 5] = [
        Fault::Missing,
        Fault::FlipBit,
        Fault::Truncate,
        Fault::Grow,
        Fault::Unreadable,
    ];

    /// An in-memory project folder: relative path and content.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct SimFolder {
        files: BTreeMap<String, Vec<u8>>,
    }

    fn sha_hex(bytes: &[u8]) -> String {
        Sha256::digest(bytes)
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect()
    }

    impl SimFolder {
        /// A folder of `count` files (one level deep, 16 to 79 bytes each) made from the seed.
        pub fn generate(seed: u64, count: usize) -> SimFolder {
            let mut rng = SimRng::new(seed);
            let mut files = BTreeMap::new();
            for index in 0..count {
                let dir = if index % 2 == 0 { "design" } else { "out" };
                let len = 16 + rng.below(64);
                let bytes = (0..len).map(|_| rng.next_u64() as u8).collect();
                files.insert(format!("{dir}/part{index:03}.bin"), bytes);
            }
            SimFolder { files }
        }

        /// The relative paths, in order.
        pub fn paths(&self) -> Vec<String> {
            self.files.keys().cloned().collect()
        }

        /// The passport that describes this folder as it is now: the files under `design/` are inputs and the rest are outputs.
        pub fn passport(&self) -> Passport {
            let reference = |(path, bytes): (&String, &Vec<u8>)| FileRef {
                path: path.clone(),
                sha256: sha_hex(bytes),
                note: String::new(),
            };
            Passport {
                project: "simulated".into(),
                created: None,
                tools: vec![Tool {
                    name: "simulator".into(),
                    version: "1".into(),
                }],
                inputs: self
                    .files
                    .iter()
                    .filter(|(path, _)| path.starts_with("design/"))
                    .map(reference)
                    .collect(),
                outputs: self
                    .files
                    .iter()
                    .filter(|(path, _)| !path.starts_with("design/"))
                    .map(reference)
                    .collect(),
                checks: vec![],
                limitations: vec![],
                pending: vec![],
            }
        }

        /// Write the whole folder under `dir` (the folders in between are made).
        pub fn materialize(&self, dir: &Path) -> io::Result<()> {
            for path in self.files.keys() {
                self.restore(dir, path)?;
            }
            Ok(())
        }

        /// Write one file again as it is in the twin; whatever stands in its place (a folder too) is removed first.
        pub fn restore(&self, dir: &Path, path: &str) -> io::Result<()> {
            let bytes = self
                .files
                .get(path)
                .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "not in the twin"))?;
            let target = dir.join(path);
            if target.is_dir() {
                fs::remove_dir_all(&target)?;
            }
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(target, bytes)
        }

        /// Break one file under `dir` with the fault; the choices (which bit, how short) come from the generator.
        pub fn inject(
            &self,
            dir: &Path,
            path: &str,
            fault: Fault,
            rng: &mut SimRng,
        ) -> io::Result<()> {
            let target = dir.join(path);
            match fault {
                Fault::Missing => fs::remove_file(target),
                Fault::FlipBit => {
                    let mut bytes = fs::read(&target)?;
                    let at = rng.below(bytes.len());
                    bytes[at] ^= 1 << rng.below(8);
                    fs::write(target, bytes)
                }
                Fault::Truncate => {
                    let bytes = fs::read(&target)?;
                    fs::write(target, &bytes[..rng.below(bytes.len())])
                }
                Fault::Grow => {
                    let mut bytes = fs::read(&target)?;
                    let extra = 1 + rng.below(8);
                    bytes.resize(bytes.len() + extra, 0);
                    fs::write(target, bytes)
                }
                Fault::Unreadable => {
                    fs::remove_file(&target)?;
                    fs::create_dir(target)
                }
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::config::Config;
        use crate::error::PassportError;
        use crate::Code;

        fn findings(folder: &SimFolder, dir: &Path) -> Vec<(Code, String)> {
            folder
                .passport()
                .verify(dir, &Config::default())
                .unwrap()
                .into_iter()
                .map(|f| (f.code, f.subject))
                .collect()
        }

        #[test]
        fn the_same_seed_gives_the_same_folder_and_another_seed_a_different_one() {
            assert_eq!(SimFolder::generate(7, 6), SimFolder::generate(7, 6));
            assert_ne!(SimFolder::generate(7, 6), SimFolder::generate(8, 6));
            assert_eq!(
                SimFolder::generate(7, 6).passport().hash(),
                SimFolder::generate(7, 6).passport().hash()
            );
        }

        #[test]
        fn a_folder_just_written_verifies_clean_and_the_passport_checks_clean() {
            let folder = SimFolder::generate(1, 8);
            let dir = tempfile::tempdir().unwrap();
            folder.materialize(dir.path()).unwrap();
            assert!(findings(&folder, dir.path()).is_empty());
            assert!(folder.passport().check(&Config::default()).is_empty());
        }

        #[test]
        fn each_fault_gives_its_own_finding_and_the_repair_brings_the_folder_back() {
            for fault in ALL_FAULTS {
                let folder = SimFolder::generate(3, 6);
                let dir = tempfile::tempdir().unwrap();
                folder.materialize(dir.path()).unwrap();
                let mut rng = SimRng::new(11);
                let path = folder.paths()[2].clone();
                folder.inject(dir.path(), &path, fault, &mut rng).unwrap();
                let expected = match fault {
                    Fault::Missing | Fault::Unreadable => Code::MissingFile,
                    _ => Code::FileMismatch,
                };
                assert_eq!(
                    findings(&folder, dir.path()),
                    vec![(expected, path.clone())],
                    "{fault:?}"
                );
                folder.restore(dir.path(), &path).unwrap();
                assert!(findings(&folder, dir.path()).is_empty(), "{fault:?}");
            }
        }

        #[test]
        fn a_seeded_campaign_finds_exactly_the_files_it_broke_and_repeats_exactly() {
            let run = |seed: u64| {
                let folder = SimFolder::generate(seed, 12);
                let dir = tempfile::tempdir().unwrap();
                folder.materialize(dir.path()).unwrap();
                let mut rng = SimRng::new(seed ^ 0xABCD);
                let mut broken = Vec::new();
                for path in folder.paths() {
                    if rng.below(3) == 0 {
                        let fault = ALL_FAULTS[rng.below(ALL_FAULTS.len())];
                        folder.inject(dir.path(), &path, fault, &mut rng).unwrap();
                        broken.push(path);
                    }
                }
                let result = findings(&folder, dir.path());
                let mut found: Vec<String> =
                    result.iter().map(|(_, subject)| subject.clone()).collect();
                found.sort();
                broken.sort();
                assert_eq!(found, broken, "seed {seed}");
                result
            };
            for seed in 1..=25 {
                assert_eq!(run(seed), run(seed), "seed {seed}");
            }
        }

        #[test]
        fn a_file_over_the_limit_stops_verify_with_an_error_and_the_default_limit_recovers() {
            let folder = SimFolder::generate(5, 4);
            let dir = tempfile::tempdir().unwrap();
            folder.materialize(dir.path()).unwrap();
            let tight = Config {
                max_hashed_bytes: 8,
                ..Config::default()
            };
            match folder.passport().verify(dir.path(), &tight) {
                Err(PassportError::TooLarge { .. }) => {}
                other => panic!("expected the limit error, got {other:?}"),
            }
            assert!(findings(&folder, dir.path()).is_empty());
        }

        #[test]
        fn an_unsafe_path_in_the_passport_is_reported_and_never_read() {
            let folder = SimFolder::generate(9, 3);
            let dir = tempfile::tempdir().unwrap();
            folder.materialize(dir.path()).unwrap();
            let mut passport = folder.passport();
            passport.outputs.push(FileRef {
                path: "../outside.bin".into(),
                sha256: "0".repeat(64),
                note: String::new(),
            });
            let found = passport.verify(dir.path(), &Config::default()).unwrap();
            assert_eq!(found.len(), 1);
            assert_eq!(found[0].code, Code::UnsafePath);
        }

        #[test]
        fn the_generator_never_stalls_on_a_zero_seed() {
            let mut rng = SimRng::new(0);
            assert_ne!(rng.next_u64(), rng.next_u64());
        }
    }
}
