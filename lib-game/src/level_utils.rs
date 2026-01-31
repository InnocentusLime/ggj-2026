use std::{path::PathBuf, str::FromStr};

use lib_asset::{Asset, LevelId, level::LevelDef};

pub fn resolve_level(s: &str) -> Option<LevelId> {
    let s = s.trim();
    LevelDef::inverse_resolve(&PathBuf::from_str(s).ok()?).ok()
}
