#[cfg(feature = "dev-env")]
use crate::DevableAsset;
use crate::level::LevelDef;
use crate::{Asset, FsResolver};
use crate::{GameCfg, asset_roots::*};
use anyhow::{anyhow, bail, ensure, Context};
use macroquad::prelude::*;
use strum::VariantArray;
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
    Screen,
}

impl Asset for Texture2D {
    type AssetId = TextureId;
    const ROOT: AssetRoot = AssetRoot::Assets;

    async fn load(_resolver: &FsResolver, path: &Path) -> anyhow::Result<Self> {
        let tex = load_texture(&path.to_string_lossy()).await?;
        Ok(tex)
    }

    fn filename(id: Self::AssetId) -> String {
        match id {
            TextureId::Objs => "objs.png",
            TextureId::Items => "items.png",
            TextureId::Mobs => "mobs.png",
            TextureId::World => "world.png",
            TextureId::Screen => panic!("file-less texture"),
        }.to_string()
    }
    
    fn inverse_resolve(filename: &Path) -> anyhow::Result<Self::AssetId> {
        TextureId::VARIANTS.iter()
            .copied()
            .find(|x| &Self::filename(*x) == filename.to_str().unwrap())
            .ok_or_else(|| anyhow!("{filename:?} does not correspond to a texture"))
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

    fn filename(id: Self::AssetId) -> String {
        match id {
            FontId::Quaver => "quaver.ttf".to_string(),
        }
    }
    
    fn inverse_resolve(_filename: &Path) -> anyhow::Result<Self::AssetId> {
        todo!()
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
)]
#[repr(transparent)]
#[serde(transparent)]
pub struct LevelId(pub UVec2);

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

const ROOM_PREFIX: &str = "room";

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

    fn filename(id: Self::AssetId) -> String {
        format!("{ROOM_PREFIX}_{}_{}.json", id.0.x, id.0.y)
    }
    
    fn inverse_resolve(filename: &Path) -> anyhow::Result<Self::AssetId> {
        let Some(filename) = filename.to_str() else {
            bail!("{filename:?} is not a valid UTF8 string");
        };
        let pieces = filename.split('_').collect::<Vec<_>>();
        ensure!(pieces.len() == 3, "filename must be 3 pieces, separated with \"_\"");
        ensure!(pieces[0] == ROOM_PREFIX, "piece 1 must be {ROOM_PREFIX:?}");
        let x = pieces[1].parse()
            .with_context(|| format!("{:?} is not a valid integer", pieces[1]))?;
        let y = pieces[2].parse()
            .with_context(|| format!("{:?} is not a valid integer", pieces[1]))?;
        Ok(LevelId(uvec2(x, y)))
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

    fn filename(_id: Self::AssetId) -> String {
        "gamecfg.json".to_string()
    }
    
    fn inverse_resolve(_filename: &Path) -> anyhow::Result<Self::AssetId> {
       bail!("Unsupported") 
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameCfgId {
    Cfg,
}

#[derive(Default, Debug, Clone, Copy, serde::Deserialize)]
pub struct PlayerAttributes {
    #[serde(default)]
    pub invisible_to_grunts: bool,
    #[serde(default)]
    pub strong_against_grunts: bool,
}

impl Asset for Vec<PlayerAttributes> {
    type AssetId = ();
    const ROOT: AssetRoot = AssetRoot::Base;

    async fn load(
        resolver: &FsResolver,
        path: &Path,
    ) -> anyhow::Result<Self> {
        use anyhow::Context;
        use macroquad::prelude::*;
        let json = load_string(path.to_str().unwrap())
            .await
            .context("loading JSON")?;
        serde_json::from_str(&json).context("decoding")
    }

    fn filename(id: Self::AssetId) -> String {
        "masks.json".to_string()
    }

    fn inverse_resolve(filename: &Path) -> anyhow::Result<Self::AssetId> {
        unimplemented!()
    }
}
