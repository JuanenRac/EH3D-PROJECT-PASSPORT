// SPDX-License-Identifier: GPL-3.0-or-later
//! The passport: its sections, the checks, the canonical hash, the comparison with the files of a folder and the comparison of two passports.

use crate::config::Config;
use crate::error::PassportError;
use crate::Code;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::io::Read;
use std::path::Path;

/// A tool and the version that was used.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Tool {
    /// The name of the tool.
    pub name: String,
    /// Its version, as the tool reports it.
    pub version: String,
}

/// A file of the project, by relative path and SHA-256.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FileRef {
    /// The path, relative to the folder of the project, with `/` separators.
    pub path: String,
    /// The SHA-256 of the content (64 lower-case hexadecimal digits).
    pub sha256: String,
    /// A free note.
    #[serde(default)]
    pub note: String,
}

/// The result of one check and the hash of its report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckRef {
    /// The name of the check (`rules`, `manufacturing`, ...).
    pub name: String,
    /// What it said (`pass`, `fail`, `inconclusive`), as text.
    pub verdict: String,
    /// The SHA-256 of its report.
    pub report_sha256: String,
}

/// The passport. Every section is required: an empty one is written as empty.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Passport {
    /// The name of the project.
    pub project: String,
    /// When the passport was made, as text supplied by the person (this crate does not read the clock).
    #[serde(default)]
    pub created: Option<String>,
    /// The tools and their versions.
    pub tools: Vec<Tool>,
    /// The files the design starts from.
    pub inputs: Vec<FileRef>,
    /// The files and packages the project produced.
    pub outputs: Vec<FileRef>,
    /// The results of the checks.
    pub checks: Vec<CheckRef>,
    /// The known limitations.
    pub limitations: Vec<String>,
    /// The decisions still pending.
    pub pending: Vec<String>,
}

/// Something wrong in a passport or between a passport and its files.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    /// The code.
    pub code: Code,
    /// What it is about (a path or a name).
    pub subject: String,
    /// What is wrong, in English.
    pub message: String,
}

/// Whether the text is 64 lower-case hexadecimal digits.
pub fn is_sha256(text: &str) -> bool {
    text.len() == 64
        && text
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

/// Whether the path is a safe relative path: not empty, not absolute, no drive, no backslash, no empty, `.` or `..` part.
pub fn is_safe_path(path: &str) -> bool {
    !path.is_empty()
        && !path.starts_with('/')
        && !path.contains('\\')
        && !path.contains(':')
        && path
            .split('/')
            .all(|part| !part.is_empty() && part != "." && part != "..")
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// The SHA-256 of a file, read in blocks, within the limit of the settings.
pub fn hash_file(path: &Path, config: &Config) -> Result<String, PassportError> {
    let io = |source| PassportError::Io {
        path: path.to_path_buf(),
        source,
    };
    if config.reject_symlinks
        && fs::symlink_metadata(path)
            .map_err(io)?
            .file_type()
            .is_symlink()
    {
        return Err(PassportError::Symlink {
            path: path.to_path_buf(),
        });
    }
    let size = fs::metadata(path).map_err(io)?.len();
    if size > config.max_hashed_bytes {
        return Err(PassportError::TooLarge {
            what: "file to hash",
        });
    }
    let file = fs::File::open(path).map_err(io)?;
    // what is read is bounded too: a file that grows after its size was looked at, or a pipe or a device that says it is empty, stops at the limit
    match hash_reader(file, config.max_hashed_bytes).map_err(io)? {
        Some(hash) => Ok(hash),
        None => Err(PassportError::TooLarge {
            what: "file to hash",
        }),
    }
}

/// The SHA-256 of what a reader gives, read in blocks; `None` as soon as it gives more than `limit` bytes.
fn hash_reader<R: Read>(mut reader: R, limit: u64) -> std::io::Result<Option<String>> {
    let mut hasher = Sha256::new();
    let mut block = vec![0u8; 64 * 1024];
    let mut total: u64 = 0;
    loop {
        let n = reader.read(&mut block)?;
        if n == 0 {
            break;
        }
        total += n as u64;
        if total > limit {
            return Ok(None);
        }
        hasher.update(&block[..n]);
    }
    Ok(Some(hex(&hasher.finalize())))
}

/// The references of the given files of a folder (sorted by path), for writing a passport. The folder is only read.
pub fn hash_files(
    base: &Path,
    paths: &[String],
    config: &Config,
) -> Result<Vec<FileRef>, PassportError> {
    let mut out = Vec::new();
    for path in paths {
        if !is_safe_path(path) {
            return Err(PassportError::UnsafePath(path.clone()));
        }
        out.push(FileRef {
            path: path.clone(),
            sha256: hash_file(&base.join(path), config)?,
            note: String::new(),
        });
    }
    out.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(out)
}

impl Passport {
    /// Read a passport from JSON, within the bounds of the configuration. A missing section or a field the format does not know is an error.
    pub fn from_json(text: &str, config: &Config) -> Result<Passport, PassportError> {
        if text.len() as u64 > config.max_file_bytes {
            return Err(PassportError::TooLarge {
                what: "passport bytes",
            });
        }
        let passport: Passport =
            serde_json::from_str(text).map_err(|e| PassportError::BadJson(e.to_string()))?;
        let biggest = [
            passport.tools.len(),
            passport.inputs.len(),
            passport.outputs.len(),
            passport.checks.len(),
            passport.limitations.len(),
            passport.pending.len(),
        ]
        .into_iter()
        .max()
        .unwrap_or(0);
        if biggest > config.max_entries {
            return Err(PassportError::TooLarge { what: "entries" });
        }
        Ok(passport)
    }

    /// The passport with every section sorted: the same content in another order is the same canonical passport.
    pub fn canonical(&self) -> Passport {
        let mut p = self.clone();
        p.tools.sort_by(|a, b| a.name.cmp(&b.name));
        p.inputs.sort_by(|a, b| a.path.cmp(&b.path));
        p.outputs.sort_by(|a, b| a.path.cmp(&b.path));
        p.checks.sort_by(|a, b| a.name.cmp(&b.name));
        p.limitations.sort();
        p.pending.sort();
        p
    }

    /// The canonical passport as JSON: the same content always gives the same bytes.
    pub fn to_json(&self) -> Result<String, PassportError> {
        serde_json::to_string_pretty(&self.canonical())
            .map_err(|e| PassportError::BadJson(e.to_string()))
    }

    /// The SHA-256 of the canonical content (lower-case hexadecimal).
    pub fn hash(&self) -> String {
        let text = serde_json::to_string(&self.canonical()).unwrap_or_default();
        hex(&Sha256::digest(text.as_bytes()))
    }

    /// What is wrong in the passport itself.
    pub fn check(&self, config: &Config) -> Vec<Finding> {
        let mut out = Vec::new();
        let f = |code, subject: &str, message: &str| Finding {
            code,
            subject: subject.to_string(),
            message: message.to_string(),
        };
        let mut names: BTreeMap<&str, usize> = BTreeMap::new();
        for t in &self.tools {
            *names.entry(t.name.as_str()).or_insert(0) += 1;
            if config.require_tool_versions && t.version.trim().is_empty() {
                out.push(f(
                    Code::MissingToolVersion,
                    &t.name,
                    "the tool has no version",
                ));
            }
        }
        for (name, count) in names {
            if count > 1 {
                out.push(f(
                    Code::Duplicate,
                    name,
                    "the tool is listed more than once",
                ));
            }
        }
        for (section, files) in [("input", &self.inputs), ("output", &self.outputs)] {
            let mut seen = BTreeMap::new();
            for file in files {
                if !is_safe_path(&file.path) {
                    out.push(f(Code::UnsafePath, &file.path, "not a safe relative path"));
                }
                if !is_sha256(&file.sha256) {
                    out.push(f(
                        Code::BadHash,
                        &file.path,
                        "not 64 lower-case hexadecimal digits",
                    ));
                }
                if seen.insert(file.path.as_str(), ()).is_some() {
                    out.push(f(
                        Code::Duplicate,
                        &file.path,
                        &format!("listed more than once among the {section}s"),
                    ));
                }
            }
        }
        for c in &self.checks {
            if !is_sha256(&c.report_sha256) {
                out.push(f(
                    Code::BadHash,
                    &c.name,
                    "the hash of the report is not 64 lower-case hexadecimal digits",
                ));
            }
        }
        out.sort_by(|a, b| (a.code, &a.subject).cmp(&(b.code, &b.subject)));
        out
    }

    /// Compare the files of a folder with the passport: a file that is missing or whose content is no longer what was recorded. The folder is only read.
    pub fn verify(&self, base: &Path, config: &Config) -> Result<Vec<Finding>, PassportError> {
        let mut out = Vec::new();
        for file in self.inputs.iter().chain(&self.outputs) {
            if !is_safe_path(&file.path) {
                out.push(Finding {
                    code: Code::UnsafePath,
                    subject: file.path.clone(),
                    message: "not a safe relative path, so it was not read".into(),
                });
                continue;
            }
            let path = base.join(&file.path);
            if !path.is_file() {
                out.push(Finding {
                    code: Code::MissingFile,
                    subject: file.path.clone(),
                    message: "the file is not in the folder".into(),
                });
                continue;
            }
            let actual = hash_file(&path, config)?;
            if actual != file.sha256 {
                out.push(Finding {
                    code: Code::FileMismatch,
                    subject: file.path.clone(),
                    message: "the content is no longer what the passport recorded".into(),
                });
            }
        }
        out.sort_by(|a, b| (a.code, &a.subject).cmp(&(b.code, &b.subject)));
        Ok(out)
    }

    /// What differs between two passports, one line each, in a fixed order.
    pub fn compare(&self, other: &Passport) -> Vec<String> {
        let (a, b) = (self.canonical(), other.canonical());
        let mut out = Vec::new();
        let tools = |p: &Passport| -> BTreeMap<String, String> {
            p.tools
                .iter()
                .map(|t| (t.name.clone(), t.version.clone()))
                .collect()
        };
        let (ta, tb) = (tools(&a), tools(&b));
        for (name, va) in &ta {
            match tb.get(name) {
                None => out.push(format!("tool removed: {name}")),
                Some(vb) if vb != va => out.push(format!("tool changed: {name} {va} -> {vb}")),
                _ => {}
            }
        }
        for name in tb.keys().filter(|n| !ta.contains_key(*n)) {
            out.push(format!("tool added: {name}"));
        }
        for (section, fa, fb) in [
            ("input", &a.inputs, &b.inputs),
            ("output", &a.outputs, &b.outputs),
        ] {
            let ma: BTreeMap<&str, &str> = fa
                .iter()
                .map(|f| (f.path.as_str(), f.sha256.as_str()))
                .collect();
            let mb: BTreeMap<&str, &str> = fb
                .iter()
                .map(|f| (f.path.as_str(), f.sha256.as_str()))
                .collect();
            for (path, ha) in &ma {
                match mb.get(path) {
                    None => out.push(format!("{section} removed: {path}")),
                    Some(hb) if hb != ha => out.push(format!("{section} changed: {path}")),
                    _ => {}
                }
            }
            for path in mb.keys().filter(|p| !ma.contains_key(*p)) {
                out.push(format!("{section} added: {path}"));
            }
        }
        let checks = |p: &Passport| -> BTreeMap<String, (String, String)> {
            p.checks
                .iter()
                .map(|c| (c.name.clone(), (c.verdict.clone(), c.report_sha256.clone())))
                .collect()
        };
        let (ca, cb) = (checks(&a), checks(&b));
        for (name, (verdict_a, report_a)) in &ca {
            match cb.get(name) {
                None => out.push(format!("check removed: {name}")),
                Some((verdict_b, report_b)) => {
                    if verdict_b != verdict_a {
                        out.push(format!("check changed: {name} {verdict_a} -> {verdict_b}"));
                    } else if report_b != report_a {
                        out.push(format!("check report changed: {name}"));
                    }
                }
            }
        }
        for name in cb.keys().filter(|n| !ca.contains_key(*n)) {
            out.push(format!("check added: {name}"));
        }
        for (section, la, lb) in [
            ("limitation", &a.limitations, &b.limitations),
            ("pending", &a.pending, &b.pending),
        ] {
            for text in la.iter().filter(|t| !lb.contains(t)) {
                out.push(format!("{section} removed: {text}"));
            }
            for text in lb.iter().filter(|t| !la.contains(t)) {
                out.push(format!("{section} added: {text}"));
            }
        }
        if a.project != b.project {
            out.push(format!("project changed: {} -> {}", a.project, b.project));
        }
        if a.created != b.created {
            out.push(String::from("created changed"));
        }
        // a safety net: two passports with different content never compare as equal
        if out.is_empty() && a.hash() != b.hash() {
            out.push(String::from("other content changed"));
        }
        out
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn a_reader_that_never_ends_stops_at_the_limit() {
        assert_eq!(
            super::hash_reader(std::io::repeat(0), 1_000_000).unwrap(),
            None
        );
        let abc = super::hash_reader(&b"abc"[..], 3).unwrap().unwrap();
        assert_eq!(
            abc,
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_eq!(super::hash_reader(&b"abcd"[..], 3).unwrap(), None);
    }

    use super::*;

    fn sha(byte: char) -> String {
        byte.to_string().repeat(64)
    }

    fn sample() -> Passport {
        Passport {
            project: "board".into(),
            created: None,
            tools: vec![
                Tool {
                    name: "phoenix".into(),
                    version: "0.1.0".into(),
                },
                Tool {
                    name: "verifier".into(),
                    version: "0.2.0".into(),
                },
            ],
            inputs: vec![FileRef {
                path: "design/board.phoenixproj".into(),
                sha256: sha('a'),
                note: String::new(),
            }],
            outputs: vec![FileRef {
                path: "out/board.eh3dproj".into(),
                sha256: sha('b'),
                note: String::new(),
            }],
            checks: vec![CheckRef {
                name: "rules".into(),
                verdict: "pass".into(),
                report_sha256: sha('c'),
            }],
            limitations: vec!["never built on a machine".into()],
            pending: vec![],
        }
    }

    fn codes(p: &Passport) -> Vec<Code> {
        p.check(&Config::default()).iter().map(|f| f.code).collect()
    }

    #[test]
    fn the_hash_of_a_fixed_passport_is_the_known_one() {
        // a fixed vector: a change of the order of the fields or of the way the content is written changes every hash that was ever quoted
        assert_eq!(
            sample().hash(),
            "1118e75bc3ac4a808d16b55a9edb99d8de12217bb5f79019815a39b83731ea13"
        );
    }

    #[test]
    fn a_clean_passport_has_no_findings() {
        assert!(codes(&sample()).is_empty());
    }

    #[test]
    fn bad_hashes_paths_duplicates_and_missing_versions_are_flagged() {
        let mut p = sample();
        p.inputs.push(FileRef {
            path: "../secret".into(),
            sha256: "xyz".into(),
            note: String::new(),
        });
        p.inputs.push(p.inputs[0].clone());
        p.tools.push(Tool {
            name: "phoenix".into(),
            version: " ".into(),
        });
        p.checks[0].report_sha256 = "ABC".into();
        let c = codes(&p);
        for expected in [
            Code::BadHash,
            Code::UnsafePath,
            Code::Duplicate,
            Code::MissingToolVersion,
        ] {
            assert!(c.contains(&expected), "{expected:?} in {c:?}");
        }
        let lax = Config {
            require_tool_versions: false,
            ..Config::default()
        };
        assert!(!p
            .check(&lax)
            .iter()
            .any(|f| f.code == Code::MissingToolVersion));
        for bad in ["", "/abs", "a\\b", "c:x", "a//b", "a/./b", "a/../b"] {
            assert!(!is_safe_path(bad), "{bad:?}");
        }
        assert!(is_safe_path("a/b.c") && is_sha256(&sha('0')) && !is_sha256(&sha('G')));
    }

    #[cfg(unix)]
    #[test]
    fn a_symbolic_link_is_refused_unless_the_settings_follow_links() {
        let work = tempfile::tempdir().unwrap();
        fs::write(work.path().join("real.bin"), b"data").unwrap();
        std::os::unix::fs::symlink(work.path().join("real.bin"), work.path().join("link.bin"))
            .unwrap();
        let config = Config::default();
        assert!(config.reject_symlinks);
        assert!(matches!(
            hash_file(&work.path().join("link.bin"), &config),
            Err(PassportError::Symlink { .. })
        ));
        assert!(hash_file(&work.path().join("real.bin"), &config).is_ok());
        let follow = Config {
            reject_symlinks: false,
            ..Config::default()
        };
        assert_eq!(
            hash_file(&work.path().join("link.bin"), &follow).unwrap(),
            hash_file(&work.path().join("real.bin"), &config).unwrap()
        );
    }

    #[test]
    fn every_section_is_required_and_unknown_fields_are_refused() {
        let config = Config::default();
        let text = sample().to_json().unwrap();
        assert_eq!(
            Passport::from_json(&text, &config).unwrap(),
            sample().canonical()
        );
        let without = text.replace("\"pending\": []", "\"unknown\": []");
        assert!(matches!(
            Passport::from_json(&without, &config),
            Err(PassportError::BadJson(_))
        ));
        let missing =
            r#"{"project":"p","tools":[],"inputs":[],"outputs":[],"checks":[],"limitations":[]}"#;
        assert!(matches!(
            Passport::from_json(missing, &config),
            Err(PassportError::BadJson(_))
        ));
        for bad in ["", "[]"] {
            assert!(matches!(
                Passport::from_json(bad, &config),
                Err(PassportError::BadJson(_))
            ));
        }
    }

    #[test]
    fn the_same_content_in_another_order_has_the_same_hash_and_bytes() {
        let a = sample();
        let mut b = sample();
        b.tools.reverse();
        b.limitations.push("z".into());
        b.limitations.pop();
        assert_eq!(a.hash(), b.hash());
        assert_eq!(a.to_json().unwrap(), b.to_json().unwrap());
        let mut c = sample();
        c.outputs[0].sha256 = sha('d');
        assert_ne!(a.hash(), c.hash());
        assert_eq!(a.hash().len(), 64);
    }

    #[test]
    fn verify_finds_the_files_that_changed_or_are_missing_and_never_writes() {
        let work = tempfile::tempdir().unwrap();
        fs::create_dir_all(work.path().join("design")).unwrap();
        fs::create_dir_all(work.path().join("out")).unwrap();
        fs::write(work.path().join("design/board.phoenixproj"), b"design").unwrap();
        fs::write(work.path().join("out/board.eh3dproj"), b"package").unwrap();
        let config = Config::default();
        let refs = hash_files(
            work.path(),
            &[
                "out/board.eh3dproj".to_string(),
                "design/board.phoenixproj".to_string(),
            ],
            &config,
        )
        .unwrap();
        assert_eq!(refs[0].path, "design/board.phoenixproj");
        let mut p = sample();
        p.inputs = vec![refs[0].clone()];
        p.outputs = vec![refs[1].clone()];
        assert!(p.verify(work.path(), &config).unwrap().is_empty());
        fs::write(work.path().join("out/board.eh3dproj"), b"changed").unwrap();
        fs::remove_file(work.path().join("design/board.phoenixproj")).unwrap();
        let found = p.verify(work.path(), &config).unwrap();
        let codes: Vec<Code> = found.iter().map(|f| f.code).collect();
        assert_eq!(codes, vec![Code::FileMismatch, Code::MissingFile]);
        assert_eq!(
            fs::read(work.path().join("out/board.eh3dproj")).unwrap(),
            b"changed"
        );
        p.inputs.push(FileRef {
            path: "../x".into(),
            sha256: sha('a'),
            note: String::new(),
        });
        assert!(p
            .verify(work.path(), &config)
            .unwrap()
            .iter()
            .any(|f| f.code == Code::UnsafePath));
    }

    #[test]
    fn hashing_files_refuses_unsafe_paths_and_big_files() {
        let work = tempfile::tempdir().unwrap();
        fs::write(work.path().join("a.bin"), b"1234").unwrap();
        let config = Config::default();
        assert!(matches!(
            hash_files(work.path(), &["../a.bin".to_string()], &config),
            Err(PassportError::UnsafePath(_))
        ));
        assert!(matches!(
            hash_files(work.path(), &["missing.bin".to_string()], &config),
            Err(PassportError::Io { .. })
        ));
        let small = Config {
            max_hashed_bytes: 2,
            ..Config::default()
        };
        assert!(matches!(
            hash_files(work.path(), &["a.bin".to_string()], &small),
            Err(PassportError::TooLarge { .. })
        ));
        // the SHA-256 of "1234"
        let refs = hash_files(work.path(), &["a.bin".to_string()], &config).unwrap();
        assert_eq!(
            refs[0].sha256,
            "03ac674216f3e15c761ee1a5e255f067953623c8b388b4459e13f978d7c846f4"
        );
    }

    #[test]
    fn two_passports_are_compared_line_by_line_in_a_fixed_order() {
        let a = sample();
        let mut b = sample();
        b.tools[0].version = "0.2.0".into();
        b.tools.push(Tool {
            name: "thermal".into(),
            version: "1".into(),
        });
        b.inputs[0].sha256 = sha('e');
        b.outputs.push(FileRef {
            path: "out/extra.gbr".into(),
            sha256: sha('f'),
            note: String::new(),
        });
        assert_eq!(
            a.compare(&b),
            vec![
                "tool changed: phoenix 0.1.0 -> 0.2.0",
                "tool added: thermal",
                "input changed: design/board.phoenixproj",
                "output added: out/extra.gbr",
            ]
        );
        assert!(a.compare(&a).is_empty());
    }

    #[test]
    fn checks_limitations_pending_project_and_date_are_compared_too() {
        let a = sample();
        let mut b = sample();
        assert!(a.compare(&b).is_empty());
        b.project = "other".into();
        b.created = Some("later".into());
        b.limitations.push("a new limit".into());
        b.pending.push("a new decision".into());
        b.checks.push(CheckRef {
            name: "extra".into(),
            verdict: "pass".into(),
            report_sha256: sha('1'),
        });
        let lines = a.compare(&b);
        for want in [
            "check added: extra",
            "limitation added: a new limit",
            "pending added: a new decision",
            "created changed",
        ] {
            assert!(lines.iter().any(|l| l == want), "{want}: {lines:?}");
        }
        assert!(
            lines.iter().any(|l| l.starts_with("project changed: ")),
            "{lines:?}"
        );
        // a check that changes its verdict, or only its report
        let c = sample();
        let mut d = c.clone();
        d.checks[0].verdict = "fail".into();
        assert_eq!(c.compare(&d), vec!["check changed: rules pass -> fail"]);
        let mut e = c.clone();
        e.checks[0].report_sha256 = sha('3');
        assert_eq!(c.compare(&e), vec!["check report changed: rules"]);
    }

    #[test]
    fn the_size_limits_apply() {
        let text = sample().to_json().unwrap();
        let small = Config {
            max_file_bytes: 10,
            ..Config::default()
        };
        assert!(matches!(
            Passport::from_json(&text, &small),
            Err(PassportError::TooLarge { .. })
        ));
        let few = Config {
            max_entries: 1,
            ..Config::default()
        };
        assert!(matches!(
            Passport::from_json(&text, &few),
            Err(PassportError::TooLarge { .. })
        ));
    }

    #[test]
    fn only_sixty_four_lower_case_hexadecimal_digits_are_a_sha256() {
        assert!(is_sha256(&sha('a')));
        assert!(is_sha256(&"0123456789abcdef".repeat(4)));
        assert!(!is_sha256(&sha('A')));
        assert!(!is_sha256(&sha('g')));
        assert!(!is_sha256(&"a".repeat(63)));
        assert!(!is_sha256(&"a".repeat(65)));
        assert!(!is_sha256(""));
        // sixty-four bytes that are thirty-two letters of two bytes each
        assert!(!is_sha256(&"\u{e9}".repeat(32)));
    }

    #[test]
    fn a_safe_path_is_relative_with_slashes_and_no_dot_parts() {
        for good in [
            "a",
            "a/b.txt",
            "design/board 1.phoenixproj",
            "a/.hidden",
            "a/b/c/d",
        ] {
            assert!(is_safe_path(good), "{good}");
        }
        for bad in [
            "", "/a", "a//b", "./a", "a/./b", "../a", "a/../b", "a/..", "a\\b", "C:/x", "a/", "a:b",
        ] {
            assert!(!is_safe_path(bad), "{bad}");
        }
    }

    #[test]
    fn making_a_passport_canonical_twice_changes_nothing_and_keeps_the_hash() {
        let mut p = sample();
        p.tools.reverse();
        p.limitations.push("another limit".into());
        p.limitations.reverse();
        let once = p.canonical();
        assert_eq!(once.canonical(), once);
        assert_eq!(p.hash(), once.hash());
    }

    #[test]
    fn a_passport_written_and_read_again_is_its_canonical_form_with_the_same_bytes() {
        let config = Config::default();
        let mut p = sample();
        p.tools.reverse();
        let text = p.to_json().unwrap();
        let again = Passport::from_json(&text, &config).unwrap();
        assert_eq!(again, p.canonical());
        assert_eq!(again.to_json().unwrap(), text);
        assert_eq!(again.hash(), p.hash());
    }

    #[test]
    fn a_text_that_is_not_a_passport_is_an_error_and_never_a_panic() {
        let config = Config::default();
        for text in [
            "",
            " ",
            "null",
            "[]",
            "{",
            "{}",
            "\u{feff}{}",
            "\"text\"",
            "12",
        ] {
            assert!(
                matches!(
                    Passport::from_json(text, &config),
                    Err(PassportError::BadJson(_))
                ),
                "{text:?}"
            );
        }
    }

    #[test]
    fn a_section_of_the_wrong_type_is_refused() {
        let config = Config::default();
        let text = r#"{"project":"p","tools":"none","inputs":[],"outputs":[],"checks":[],"limitations":[],"pending":[]}"#;
        assert!(matches!(
            Passport::from_json(text, &config),
            Err(PassportError::BadJson(_))
        ));
        let text = r#"{"project":"p","tools":[],"inputs":[],"outputs":[],"checks":[],"limitations":[1],"pending":[]}"#;
        assert!(matches!(
            Passport::from_json(text, &config),
            Err(PassportError::BadJson(_))
        ));
    }

    #[test]
    fn a_passport_with_every_section_empty_is_valid_and_has_a_stable_hash() {
        let config = Config::default();
        let text = r#"{"project":"p","tools":[],"inputs":[],"outputs":[],"checks":[],"limitations":[],"pending":[]}"#;
        let p = Passport::from_json(text, &config).unwrap();
        assert_eq!(p.created, None);
        assert_eq!(p.hash(), Passport::from_json(text, &config).unwrap().hash());
        assert_eq!(p.hash().len(), 64);
    }

    #[test]
    fn the_most_entries_of_one_section_is_read_and_one_more_is_not() {
        let mut p = sample();
        p.limitations = (0..3).map(|n| format!("limit {n}")).collect();
        let text = p.to_json().unwrap();
        let exact = Config {
            max_entries: 3,
            ..Config::default()
        };
        assert!(Passport::from_json(&text, &exact).is_ok());
        let fewer = Config {
            max_entries: 2,
            ..Config::default()
        };
        assert!(matches!(
            Passport::from_json(&text, &fewer),
            Err(PassportError::TooLarge { .. })
        ));
    }

    #[test]
    fn a_change_of_the_name_of_the_project_or_of_one_hash_changes_the_hash() {
        let base = sample();
        let mut renamed = sample();
        renamed.project = "other".into();
        let mut flipped = sample();
        flipped.outputs[0].sha256 = sha('d');
        assert_ne!(base.hash(), renamed.hash());
        assert_ne!(base.hash(), flipped.hash());
        assert_ne!(renamed.hash(), flipped.hash());
    }

    #[test]
    fn a_tool_that_is_removed_or_a_check_that_changes_its_verdict_is_said() {
        let a = sample();
        let mut b = sample();
        b.tools.remove(0);
        b.checks[0].verdict = "fail".into();
        assert_eq!(
            a.compare(&b),
            vec!["tool removed: phoenix", "check changed: rules pass -> fail"]
        );
        let mut c = sample();
        c.checks[0].report_sha256 = sha('9');
        assert_eq!(a.compare(&c), vec!["check report changed: rules"]);
    }

    #[test]
    fn comparing_in_one_direction_and_in_the_other_swaps_the_added_and_the_removed() {
        let a = sample();
        let mut b = sample();
        b.pending.push("choose the board house".into());
        assert_eq!(a.compare(&b), vec!["pending added: choose the board house"]);
        assert_eq!(
            b.compare(&a),
            vec!["pending removed: choose the board house"]
        );
    }

    #[test]
    fn the_order_of_the_sections_does_not_make_two_passports_differ() {
        let a = sample();
        let mut b = sample();
        b.tools.reverse();
        b.limitations.reverse();
        assert!(a.compare(&b).is_empty());
        assert!(b.compare(&a).is_empty());
        assert_eq!(a.hash(), b.hash());
    }
}
