use super::Codex;
use bevy_asset::{io::Reader, AssetLoader, LoadContext};
use bevy_reflect::TypePath;
use serde::de::{Deserialize, DeserializeSeed, Deserializer, MapAccess, Visitor};
use std::{fmt, marker::PhantomData};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    IOError(#[from] std::io::Error),
    #[error(transparent)]
    Utf8Error(#[from] std::str::Utf8Error),
    #[error(transparent)]
    TomlError(#[from] toml::de::Error),
}

/// Data that is contained in a codex.
pub trait CodexSource: Send + Sync + 'static {
    /// Specifices the file extension for codex.
    const EXTENSION: &'static str;
}

/// Defines how an asset is processed from some raw definition using the LoadContext.
pub trait FromWithLoadContext<T> {
    fn from_with_load_context(raw: T, load_context: &mut LoadContext) -> Self;
}

impl<T> FromWithLoadContext<T> for T {
    fn from_with_load_context(raw: T, _load_context: &mut LoadContext) -> Self {
        raw
    }
}

/// Deserializer for Codex that keeps the LoadContext as state.
struct CodexDeserializer<'a, 'c, RawEntry, Entry> {
    load_context: &'a mut LoadContext<'c>,
    _phantom: PhantomData<(RawEntry, Entry)>,
}

impl<'a, 'c, RawEntry, Entry> CodexDeserializer<'a, 'c, RawEntry, Entry> {
    pub fn with_load_context(load_context: &'a mut LoadContext<'c>) -> Self {
        Self {
            load_context,
            _phantom: PhantomData,
        }
    }
}

impl<'de, RawEntry, Entry> DeserializeSeed<'de> for CodexDeserializer<'_, '_, RawEntry, Entry>
where
    RawEntry: Deserialize<'de>,
    Entry: std::fmt::Debug + TypePath + Send + Sync + FromWithLoadContext<RawEntry>,
{
    type Value = Codex<Entry>;
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(CodexVisitor::<RawEntry, Entry>::with_load_context(
            self.load_context,
        ))
    }
}

/// Visitor that modifies how a `Codex` is deserialized from a map like structure by replacing the
/// string keys with the hashed `Id` identifiers, and the value by converting from the raw form to
/// the final entry value type.
struct CodexVisitor<'a, 'c, RawEntry, Entry> {
    load_context: &'a mut LoadContext<'c>,
    _phantom: PhantomData<(RawEntry, Entry)>,
}

impl<'a, 'c, RawEntry, Entry> CodexVisitor<'a, 'c, RawEntry, Entry> {
    fn with_load_context(load_context: &'a mut LoadContext<'c>) -> Self {
        Self {
            load_context,
            _phantom: PhantomData,
        }
    }
}

impl<'de, RawEntry, Entry> Visitor<'de> for CodexVisitor<'_, '_, RawEntry, Entry>
where
    RawEntry: Deserialize<'de>,
    Entry: fmt::Debug + TypePath + Send + Sync + FromWithLoadContext<RawEntry>,
{
    type Value = Codex<Entry>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("codex")
    }

    fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
    where
        M: MapAccess<'de>,
    {
        let mut codex_builder = Codex::builder_with_capacity(access.size_hint().unwrap_or(0));

        while let Some((key, value)) = access.next_entry::<String, RawEntry>()? {
            codex_builder = codex_builder.add(
                key.as_str(),
                Entry::from_with_load_context(value, self.load_context),
            );
        }

        Ok(codex_builder.build())
    }
}

/// Loads codex assets from TOML-files
pub struct CodexLoader<RawEntry, Entry = RawEntry> {
    _phantom_data: PhantomData<(RawEntry, Entry)>,
}

impl<RawEntry, Entry> Default for CodexLoader<RawEntry, Entry> {
    fn default() -> Self {
        Self {
            _phantom_data: PhantomData,
        }
    }
}

impl<RawEntry, Entry> AssetLoader for CodexLoader<RawEntry, Entry>
where
    Entry: CodexSource + std::fmt::Debug + TypePath + Send + Sync + FromWithLoadContext<RawEntry>,
    RawEntry: Send + Sync + for<'de> Deserialize<'de> + 'static,
{
    type Asset = Codex<Entry>;
    type Settings = ();
    type Error = Error;

    fn extensions(&self) -> &[&str] {
        &[Entry::EXTENSION]
    }

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let strdata = std::str::from_utf8(&bytes)?;
        let codex: Self::Asset = CodexDeserializer::with_load_context(load_context)
            .deserialize(toml::de::Deserializer::new(strdata))?;
        Ok(codex)
    }
}
