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
