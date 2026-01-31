#[cfg(feature = "dev-env")]
use crate::DevableAsset;
use crate::level::LevelDef;
use crate::{Asset, FsResolver};
use crate::{GameCfg, asset_roots::*};
use macroquad::prelude::*;
use std::path::Path;

#[derive(
    Default,
    Debug,
    Clone,
    Copy,
    serde::Serialize,
    serde::Deserialize,
    PartialEq,
    Eq,
    Hash,
    strum::IntoStaticStr,
    strum::VariantArray,
)]
pub enum TextureId {
    Objs,
    Items,
    Mobs,
    #[default]
    World,
}

impl Asset for Texture2D {
    type AssetId = TextureId;
    const ROOT: AssetRoot = AssetRoot::Assets;

    async fn load(_resolver: &FsResolver, path: &Path) -> anyhow::Result<Self> {
        let tex = load_texture(&path.to_string_lossy()).await?;
        Ok(tex)
    }

    fn filename(id: Self::AssetId) -> &'static str {
        match id {
            TextureId::Objs => "objs.png",
            TextureId::Items => "items.png",
            TextureId::Mobs => "mobs.png",
            TextureId::World => "world.png",
        }
    }
}

#[derive(
    Debug,
    Clone,
    Copy,
    serde::Serialize,
    serde::Deserialize,
    PartialEq,
    Eq,
    Hash,
    strum::IntoStaticStr,
    strum::VariantArray,
)]
pub enum FontId {
    Quaver,
}

impl Asset for Font {
    type AssetId = FontId;
    const ROOT: AssetRoot = AssetRoot::Assets;

    async fn load(_resolver: &FsResolver, path: &Path) -> anyhow::Result<Self> {
        let font = load_ttf_font(&path.to_string_lossy()).await?;
        Ok(font)
    }

    fn filename(id: Self::AssetId) -> &'static str {
        match id {
            FontId::Quaver => "quaver.ttf",
        }
    }
}

#[derive(
    Debug,
    Clone,
    Copy,
    serde::Serialize,
    serde::Deserialize,
    PartialEq,
    Eq,
    Hash,
    strum::IntoStaticStr,
    strum::VariantArray,
)]
pub enum LevelId {
    TestRoom,
}

#[cfg(feature = "dev-env")]
impl DevableAsset for LevelDef {
    fn load_dev(resolver: &FsResolver, path: &Path) -> anyhow::Result<Self> {
        use crate::level::tiled_load;
        use std::path::PathBuf;

        let mut filename: PathBuf = path.file_name().unwrap().into();
        filename.set_extension("tmx");
        let tiled_path = resolver.get_path(AssetRoot::TiledProjectRoot, filename);
        tiled_load::load_level(resolver, tiled_path)
    }
}

impl Asset for LevelDef {
    type AssetId = LevelId;
    const ROOT: AssetRoot = AssetRoot::Levels;

    #[cfg(feature = "dev-env")]
    async fn load(resolver: &FsResolver, path: &Path) -> anyhow::Result<LevelDef> {
        Self::load_dev(resolver, path)
    }

    #[cfg(not(feature = "dev-env"))]
    async fn load(_resolver: &FsResolver, path: &Path) -> anyhow::Result<LevelDef> {
        use anyhow::Context;
        use macroquad::prelude::*;
        let json = load_string(path.to_str().unwrap())
            .await
            .context("loading JSON")?;
        serde_json::from_str(&json).context("decoding")
    }

    fn filename(id: Self::AssetId) -> &'static str {
        match id {
            LevelId::TestRoom => "test_room.json",
        }
    }
}

impl Asset for GameCfg {
    type AssetId = GameCfgId;
    const ROOT: AssetRoot = AssetRoot::Base;

    async fn load(_resolver: &FsResolver, path: &Path) -> anyhow::Result<Self> {
        use anyhow::Context;
        use macroquad::prelude::*;
        let json = load_string(path.to_str().unwrap())
            .await
            .context("loading JSON")?;
        serde_json::from_str(&json).context("decoding")
    }

    fn filename(_id: Self::AssetId) -> &'static str {
        "gamecfg.json"
    }
}

#[derive(Debug, Clone, Copy, strum::VariantArray, strum::IntoStaticStr, PartialEq, Eq, Hash)]
pub enum GameCfgId {
    Cfg,
}
