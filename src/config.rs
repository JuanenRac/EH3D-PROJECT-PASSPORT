// SPDX-License-Identifier: GPL-3.0-or-later
//! The settings of the library. Every tuned value is a line of the settings file; a missing line keeps the conservative default.

use crate::error::PassportError;

/// The settings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    /// The most bytes a passport file may have.
    pub max_file_bytes: u64,
    /// The most entries one section may hold.
    pub max_entries: usize,
    /// The largest file `verify` and `hashfiles` read, in bytes.
    pub max_hashed_bytes: u64,
    /// Every tool must have a version.
    pub require_tool_versions: bool,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            max_file_bytes: 64 * 1024 * 1024,
            max_entries: 100_000,
            max_hashed_bytes: 4 * 1024 * 1024 * 1024,
            require_tool_versions: true,
        }
    }
}

fn flag(text: &str) -> Option<bool> {
    match text {
        "true" | "on" | "yes" => Some(true),
        "false" | "off" | "no" => Some(false),
        _ => None,
    }
}

impl Config {
    /// Parse a settings file (`key = value` lines, `#` comments).
    pub fn parse(text: &str) -> Result<Config, PassportError> {
        let mut config = Config::default();
        for (index, raw) in text.lines().enumerate() {
            let line = index + 1;
            let raw = raw.trim();
            if raw.is_empty() || raw.starts_with('#') {
                continue;
            }
            let (key, value) = raw
                .split_once('=')
                .ok_or(PassportError::ConfigSyntax { line })?;
            let (key, value) = (key.trim(), value.trim());
            let bad = || PassportError::ConfigBadValue {
                line,
                key: key.to_string(),
            };
            match key {
                "max_file_bytes" => config.max_file_bytes = value.parse().map_err(|_| bad())?,
                "max_hashed_bytes" => config.max_hashed_bytes = value.parse().map_err(|_| bad())?,
                "max_entries" => {
                    let n: usize = value.parse().map_err(|_| bad())?;
                    if n == 0 {
                        return Err(bad());
                    }
                    config.max_entries = n;
                }
                "require_tool_versions" => {
                    config.require_tool_versions = flag(value).ok_or_else(bad)?
                }
                _ => {
                    return Err(PassportError::ConfigUnknownKey {
                        line,
                        key: key.to_string(),
                    })
                }
            }
        }
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_conservative() {
        assert!(Config::parse("").unwrap().require_tool_versions);
    }

    #[test]
    fn lines_change_values_and_bad_lines_are_refused() {
        let c = Config::parse("# c\nrequire_tool_versions = off\nmax_entries = 7\n").unwrap();
        assert!(!c.require_tool_versions);
        assert_eq!(c.max_entries, 7);
        assert!(matches!(
            Config::parse("x\n"),
            Err(PassportError::ConfigSyntax { line: 1 })
        ));
        assert!(matches!(
            Config::parse("a = 1\n"),
            Err(PassportError::ConfigUnknownKey { line: 1, .. })
        ));
        for bad in [
            "max_entries = 0\n",
            "require_tool_versions = maybe\n",
            "max_hashed_bytes = big\n",
        ] {
            assert!(
                matches!(
                    Config::parse(bad),
                    Err(PassportError::ConfigBadValue { line: 1, .. })
                ),
                "{bad:?}"
            );
        }
    }
}
