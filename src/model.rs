use std::collections::HashMap;
use std::fmt::Display;
use std::str;

use fuzzy_matcher::FuzzyMatcher;
use serde::{Deserialize, Serialize};

/// ```json
/// {
///   "id": "http",
///   "name": "http",
///   "version": "0.0.1",
///   "description": "Highlights .http files",
///   "authors": [
///     "tylerhanson921@gmail.com"
///   ],
///   "repository": "https://github.com/tie304/zed-http",
///   "schema_version": 1,
///   "wasm_api_version": "0.0.6",
///   "provides": [],
///   "published_at": "2024-08-14T15:18:43Z",
///   "download_count": 32716
/// }
/// ```
#[derive(Debug, Serialize, Deserialize)]
pub struct ExtensionManifest {
    pub id: String,
    pub name: String,
    pub version: semver::Version,
    pub schema_version: u16,
    pub provides: Vec<String>,
    pub download_count: u64,
    pub wasm_api_version: Option<semver::Version>,
    #[serde(flatten)]
    pub rest: HashMap<String, serde_json::Value>,
}

impl Display for ExtensionManifest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.id, self.version)
    }
}

impl ExtensionManifest {
    pub fn check_wasm_api_version(&self, min: &semver::Version, max: &semver::Version) -> bool {
        if let Some(wasm_api_version) = &self.wasm_api_version {
            return wasm_api_version >= min && wasm_api_version <= max;
        }
        true
    }

    pub const fn check_schema_version(&self, min: u16, max: u16) -> bool {
        if self.schema_version < min || self.schema_version > max {
            return false;
        }
        true
    }

    pub fn match_filter(&self, filter: &str) -> bool {
        // TODO: check for zed.dev heuristic
        let matcher = fuzzy_matcher::skim::SkimMatcherV2::default();
        matcher.fuzzy_match(&self.name, filter).is_some()
    }
}

pub const DEFAULT_ASSETS: &[&str] = &[
    "zed-linux-aarch64",
    "zed-linux-x86_64",
    "zed-macos-aarch64",
    "zed-macos-x86_64",
    "zed-windows-aarch64",
    "zed-windows-x86_64",
    "zed-remote-server-linux-aarch64",
    "zed-remote-server-linux-x86_64",
    "zed-remote-server-macos-aarch64",
    "zed-remote-server-macos-x86_64",
];

#[derive(Debug, Clone)]
pub struct Asset {
    pub name: String,
    pub os: String,
    pub arch: String,
}

impl Asset {
    pub fn filename(&self) -> String {
        if self.name != "zed" || (self.os != "windows" && self.os != "macos") {
            let mut f = self.to_string();
            f.push_str(".tar.gz");
            return f;
        }

        // Only windows and macos zed executable have different naming scheme
        match self.os.as_str() {
            "windows" => format!("Zed-{}.exe", self.arch),
            "macos" => format!("Zed-{}.dmg", self.arch),
            os => panic!("unsupported os {os}"),
        }
    }
}

impl str::FromStr for Asset {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let v: Vec<_> = s.rsplitn(3, '-').collect();
        if v.len() != 3 {
            anyhow::bail!("Asset format must be <asset>-<os>-<arch>");
        }
        Ok(Self {
            name: v[2].to_string(),
            os: v[1].to_string(),
            arch: v[0].to_string(),
        })
    }
}

impl Display for Asset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}-{}", self.name, self.os, self.arch)
    }
}

impl<'de> Deserialize<'de> for Asset {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        str::FromStr::from_str(&s).map_err(serde::de::Error::custom)
    }
}
