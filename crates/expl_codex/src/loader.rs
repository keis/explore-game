use super::Codex;
use bevy_asset::{io::Reader, AssetLoader, AsyncReadExt, BoxedFuture, LoadContext};
use bevy_reflect::TypePath;
use serde::{
    de::{Deserializer, MapAccess, Visitor},
    Deserialize,
};
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

/// Visitor that modifies how a `Codex` is deserialized from a map like structure by replacing the
/// string keys with the hashed `Id` identifiers
struct CodexVisitor<Entry> {
    _phantom: PhantomData<Entry>,
}

impl<Entry> Default for CodexVisitor<Entry> {
    fn default() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<'de, Entry> Visitor<'de> for CodexVisitor<Entry>
where
    Entry: fmt::Debug + TypePath + Send + Sync + Deserialize<'de>,
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

        while let Some((key, value)) = access.next_entry::<String, _>()? {
            codex_builder = codex_builder.add(key.as_str(), value);
        }

        Ok(codex_builder.build())
    }
}

impl<'de, Entry> Deserialize<'de> for Codex<Entry>
where
    Entry: std::fmt::Debug + TypePath + Send + Sync + Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(CodexVisitor::default())
    }
}

/// Specifices the file extension for codex of this entry type.
pub trait CodexSource: Send + Sync + 'static {
    const EXTENSION: &'static str;
}

/// Loads codex assets from TOML-files
pub struct CodexLoader<Entry> {
    _phantom_data: PhantomData<Entry>,
}

impl<Entry> Default for CodexLoader<Entry> {
    fn default() -> Self {
        Self {
            _phantom_data: PhantomData,
        }
    }
}

impl<T> AssetLoader for CodexLoader<T>
where
    T: CodexSource + std::fmt::Debug + TypePath + Send + Sync + for<'de> Deserialize<'de>,
{
    type Asset = Codex<T>;
    type Settings = ();
    type Error = Error;

    fn extensions(&self) -> &[&str] {
        &[T::EXTENSION]
    }

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _settings: &'a (),
        _load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;
            let strdata = std::str::from_utf8(&bytes)?;
            let codex: Self::Asset = toml::de::from_str(strdata)?;
            Ok(codex)
        })
    }
}
