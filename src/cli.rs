// SPDX-License-Identifier: GPL-3.0-or-later
//! The command line. It is a thin layer: parsing the arguments and printing; the checks live in the other modules.
//!
//! Exit codes: 0 everything is as expected, 1 findings or differences, 2 the input or the arguments are unusable.

use crate::config::Config;
use crate::error::PassportError;
use crate::passport::{hash_files, Finding, Passport};
use std::fs;
use std::io::Write;
use std::path::Path;

const USAGE: &str = "\
eh3d-project-passport [--config FILE] <command>

  check <passport.json>                    list what is wrong in the passport: bad hashes, unsafe paths, duplicates, tools without a version
  verify <passport.json> <folder>          compare the files of the folder with the passport (the folder is only read)
  hash <passport.json>                     print the SHA-256 of the canonical passport
  compare <a.json> <b.json>                print what differs between two passports
  hashfiles <folder> <path>...             print the references (path and SHA-256) of files of the folder, for writing a passport

Exit codes: 0 nothing found or equal, 1 findings or different, 2 unusable input.
";

fn read_text(path: &Path, limit: u64) -> Result<String, PassportError> {
    let io = |source| PassportError::Io {
        path: path.to_path_buf(),
        source,
    };
    if fs::metadata(path).map_err(io)?.len() > limit {
        return Err(PassportError::TooLarge { what: "input file" });
    }
    fs::read_to_string(path).map_err(io)
}

fn print_findings(findings: &[Finding], out: &mut dyn Write) {
    for f in findings {
        let _ = writeln!(out, "{} [{}]: {}", f.code.as_str(), f.subject, f.message);
    }
    let _ = writeln!(out, "# {} findings", findings.len());
}

/// Run the command line with the given arguments (without the program name). Returns the exit code.
pub fn run(args: &[String], out: &mut dyn Write, err: &mut dyn Write) -> i32 {
    match dispatch(args, out) {
        Ok(code) => code,
        Err(None) => {
            let _ = err.write_all(USAGE.as_bytes());
            2
        }
        Err(Some(error)) => {
            let _ = writeln!(err, "error: {error}");
            2
        }
    }
}

fn dispatch(args: &[String], out: &mut dyn Write) -> Result<i32, Option<PassportError>> {
    let mut args = args.to_vec();
    let mut config = Config::default();
    if args.first().map(String::as_str) == Some("--config") {
        if args.len() < 3 {
            return Err(None);
        }
        config = Config::parse(&read_text(Path::new(&args[1]), 1024 * 1024).map_err(Some)?)
            .map_err(Some)?;
        args.drain(..2);
    }
    let load = |path: &String| -> Result<Passport, Option<PassportError>> {
        let text = read_text(Path::new(path), config.max_file_bytes).map_err(Some)?;
        Passport::from_json(&text, &config).map_err(Some)
    };
    match args
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        ["check", _] => {
            let findings = load(&args[1])?.check(&config);
            print_findings(&findings, out);
            Ok(if findings.is_empty() { 0 } else { 1 })
        }
        ["verify", _, folder] => {
            let findings = load(&args[1])?
                .verify(Path::new(folder), &config)
                .map_err(Some)?;
            print_findings(&findings, out);
            Ok(if findings.is_empty() { 0 } else { 1 })
        }
        ["hash", _] => {
            let _ = writeln!(out, "{}", load(&args[1])?.hash());
            Ok(0)
        }
        ["compare", _, _] => {
            let lines = load(&args[1])?.compare(&load(&args[2])?);
            for line in &lines {
                let _ = writeln!(out, "{line}");
            }
            Ok(if lines.is_empty() { 0 } else { 1 })
        }
        ["hashfiles", folder, paths @ ..] if !paths.is_empty() => {
            let paths: Vec<String> = paths.iter().map(|p| p.to_string()).collect();
            let refs = hash_files(Path::new(folder), &paths, &config).map_err(Some)?;
            let text = serde_json::to_string_pretty(&refs)
                .map_err(|e| Some(PassportError::BadJson(e.to_string())))?;
            let _ = writeln!(out, "{text}");
            Ok(0)
        }
        _ => Err(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn call(list: &[&str]) -> (i32, String, String) {
        let args: Vec<String> = list.iter().map(|s| s.to_string()).collect();
        let (mut out, mut err) = (Vec::new(), Vec::new());
        let code = run(&args, &mut out, &mut err);
        (
            code,
            String::from_utf8(out).unwrap(),
            String::from_utf8(err).unwrap(),
        )
    }

    fn passport_for(hash: &str) -> String {
        format!(
            r#"{{"project":"board","tools":[{{"name":"phoenix","version":"0.1.0"}}],
            "inputs":[{{"path":"a.txt","sha256":"{hash}"}}],"outputs":[],"checks":[],
            "limitations":["never built"],"pending":[]}}"#
        )
    }

    #[test]
    fn hashfiles_then_check_verify_hash_and_compare() {
        let work = tempfile::tempdir().unwrap();
        fs::write(work.path().join("a.txt"), b"1234").unwrap();
        let folder = work.path().to_str().unwrap();
        let (code, text, _) = call(&["hashfiles", folder, "a.txt"]);
        assert_eq!(code, 0);
        assert!(text.contains("03ac674216f3e15c761ee1a5e255f067953623c8b388b4459e13f978d7c846f4"));
        let good = work.path().join("good.json");
        let other = work.path().join("other.json");
        fs::write(
            &good,
            passport_for("03ac674216f3e15c761ee1a5e255f067953623c8b388b4459e13f978d7c846f4"),
        )
        .unwrap();
        fs::write(&other, passport_for(&"0".repeat(64))).unwrap();
        let (good, other) = (good.to_str().unwrap(), other.to_str().unwrap());
        assert_eq!(call(&["check", good]).0, 0);
        assert_eq!(call(&["verify", good, folder]).0, 0);
        let (code, text, _) = call(&["verify", other, folder]);
        assert_eq!(code, 1);
        assert!(text.contains("PHOENIX-PASS-005"), "{text}");
        let (code, hash, _) = call(&["hash", good]);
        assert_eq!((code, hash.trim().len()), (0, 64));
        assert_eq!(call(&["compare", good, good]).0, 0);
        let (code, text, _) = call(&["compare", good, other]);
        assert_eq!(code, 1);
        assert!(text.contains("input changed: a.txt"), "{text}");
    }

    #[test]
    fn bad_input_and_bad_arguments_exit_2() {
        let work = tempfile::tempdir().unwrap();
        let broken = work.path().join("broken.json");
        fs::write(&broken, "{").unwrap();
        let (code, _, err) = call(&["check", broken.to_str().unwrap()]);
        assert_eq!(code, 2);
        assert!(err.contains("cannot be read"));
        assert_eq!(call(&[]).0, 2);
        assert_eq!(call(&["verify", "x.json"]).0, 2);
        assert_eq!(call(&["hashfiles", "."]).0, 2);
        assert_eq!(call(&["hashfiles", ".", "../x"]).0, 2);
        assert_eq!(call(&["check", "no-such-file.json"]).0, 2);
        assert_eq!(call(&["--config", "x"]).0, 2);
    }

    #[test]
    fn a_settings_file_changes_the_rule_about_tool_versions() {
        let work = tempfile::tempdir().unwrap();
        let p = work.path().join("p.json");
        let cfg = work.path().join("c.cfg");
        fs::write(
            &p,
            r#"{"project":"b","tools":[{"name":"phoenix","version":""}],"inputs":[],"outputs":[],"checks":[],"limitations":[],"pending":[]}"#,
        )
        .unwrap();
        fs::write(&cfg, "require_tool_versions = off\n").unwrap();
        assert_eq!(call(&["check", p.to_str().unwrap()]).0, 1);
        assert_eq!(
            call(&[
                "--config",
                cfg.to_str().unwrap(),
                "check",
                p.to_str().unwrap()
            ])
            .0,
            0
        );
    }
}
