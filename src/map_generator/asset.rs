use crate::{terrain::Terrain, ExplError};
use bevy::{
    asset::{io::Reader, AssetLoader, AsyncReadExt, BoxedFuture, LoadContext},
    prelude::*,
    reflect::TypePath,
};
use expl_codex::{Codex, Id};
use expl_hexgrid::{layout::HexagonalGridLayout, Grid};
use expl_wfc::{
    tile::{extract_tiles, standard_tile_transforms},
    util::{wrap_grid, LoadGridWith},
    Template,
};
use std::{collections::HashMap, sync::Arc};

#[derive(Resource)]
pub struct MapTemplateAsset(pub Handle<MapTemplate>);

#[derive(Clone, Asset, TypePath, Deref)]
pub struct MapTemplate(Arc<Template<Id<Terrain>>>);

#[derive(Default)]
pub struct TemplateLoader;

impl AssetLoader for TemplateLoader {
    type Asset = MapTemplate;
    type Settings = ();
    type Error = ExplError;

    fn extensions(&self) -> &[&str] {
        &["template.txt"]
    }

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _settings: &'a Self::Settings,
        load_context: &'a mut LoadContext<'_>,
    ) -> BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let terrain_codex: Codex<Terrain> = load_context
                .load_direct("codex/default.terrain.toml")
                .await
                .map_err(Box::new)?
                .take()
                .ok_or(ExplError::MissingCodex)?;
            let terrain_lookup: HashMap<char, Id<Terrain>> = terrain_codex
                .iter()
                .map(|(id, t)| (t.symbol, *id))
                .collect();
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;
            let grid =
                Grid::<HexagonalGridLayout, Id<Terrain>>::load_with(&mut bytes.as_slice(), |c| {
                    terrain_lookup.get(&c).copied().ok_or(0)
                })?;
            let wrapped_grid = wrap_grid(grid);
            let transforms = standard_tile_transforms();
            let template = Template::from_tiles(extract_tiles(&wrapped_grid, &transforms));

            Ok(MapTemplate(template.into()))
        })
    }
}
