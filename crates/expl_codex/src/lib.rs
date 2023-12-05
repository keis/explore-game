use bevy_asset::Asset;
use bevy_reflect::{Reflect, ReflectDeserialize, ReflectSerialize, TypePath};
use serde::{Deserialize, Serialize};
use std::{
    cmp,
    collections::HashMap,
    fmt,
    hash::{Hash, Hasher},
    marker::PhantomData,
    ops::Index,
    sync::Arc,
};

mod loader;
pub use loader::{CodexLoader, CodexSource, Error};

/// Identifier with a phantom binding to a specific type.
///
/// The id is created from a string tag by computing a hash which is also used to define equality
/// ```
/// use expl_codex::Id;
///
/// struct Animal;
///
/// let id: Id<Animal> = Id::from_tag("horse");
/// assert_eq!(id, Id::from_tag("horse"));
/// ```
///
/// A type parameter is used to tag the id to avoid mixing different types of identifiers in the
/// code.
/// ```compile_fail
/// use expl_codex::Id;
///
/// struct Animal;
/// struct Transport;
///
/// let animal_id: Id<Animal> = Id::from_tag("horse");
/// let transport_id: Id<Transport> = animal_id;
/// ```
#[derive(Reflect, Serialize, Deserialize)]
#[reflect(Hash, PartialEq, Serialize, Deserialize)]
pub struct Id<T> {
    value: u64,

    #[reflect(ignore)]
    #[serde(skip)]
    _phantom: PhantomData<T>,
}

// Custom impl to provide compact debug formatting.
impl<T> fmt::Debug for Id<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Id").field(&self.value).finish()
    }
}

impl<T> Copy for Id<T> {}

impl<T> Clone for Id<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Default for Id<T> {
    fn default() -> Self {
        Self {
            value: 0,
            _phantom: PhantomData,
        }
    }
}

impl<T> Eq for Id<T> {}

impl<T> PartialEq for Id<T> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl<T> Ord for Id<T> {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.value.cmp(&other.value)
    }
}

impl<T> PartialOrd for Id<T> {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Hash for Id<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}

impl<T> Id<T> {
    const HASH_P: u64 = 53;
    const HASH_M: u64 = 1_000_000_009;

    pub const fn new(value: u64) -> Self {
        Self {
            value,
            _phantom: PhantomData,
        }
    }

    // Create identifier from a string tag by hashing
    pub const fn from_tag(tag: &str) -> Self {
        Self::new(Self::hash_bytes(tag.as_bytes()))
    }

    /// A polynomial rolling hash function
    const fn hash_bytes(bytes: &[u8]) -> u64 {
        let len = bytes.len();
        let mut value = 0;
        let mut ppow = 1;
        let mut i = 0;
        while i < len {
            value = (value + (bytes[i] as u64 + 1) * ppow) % Self::HASH_M;
            ppow = (ppow * Self::HASH_P) % Self::HASH_M;
            i += 1;
        }
        value
    }
}

/// Defines a codex of data entries identified by `Id<Entry>`
#[derive(Asset, TypePath, Clone, Debug)]
pub struct Codex<Entry>
where
    Entry: fmt::Debug + TypePath + Send + Sync,
{
    lookup: Arc<HashMap<Id<Entry>, Entry>>,
}

pub struct CodexBuilder<Entry>
where
    Entry: fmt::Debug + TypePath + Send + Sync,
{
    lookup: HashMap<Id<Entry>, Entry>,
}

impl<Entry> CodexBuilder<Entry>
where
    Entry: fmt::Debug + TypePath + Send + Sync,
{
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            lookup: HashMap::with_capacity(capacity),
        }
    }

    pub fn add(mut self, tag: &str, entry: Entry) -> CodexBuilder<Entry> {
        self.lookup.insert(Id::from_tag(tag), entry);
        self
    }

    pub fn build(self) -> Codex<Entry> {
        Codex::new(self.lookup)
    }
}

impl<Entry> Codex<Entry>
where
    Entry: fmt::Debug + TypePath + Send + Sync,
{
    pub fn new(lookup: HashMap<Id<Entry>, Entry>) -> Self {
        Self {
            lookup: Arc::new(lookup),
        }
    }

    pub fn builder_with_capacity(capacity: usize) -> CodexBuilder<Entry> {
        CodexBuilder::with_capacity(capacity)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&Id<Entry>, &Entry)> {
        self.lookup.iter()
    }
}

impl<Entry> Index<&Id<Entry>> for Codex<Entry>
where
    Entry: fmt::Debug + TypePath + Send + Sync,
{
    type Output = Entry;

    fn index(&self, index: &Id<Entry>) -> &Self::Output {
        &self.lookup[index]
    }
}

impl<'a, Entry> FromIterator<(&'a str, Entry)> for Codex<Entry>
where
    Entry: fmt::Debug + TypePath + Send + Sync,
{
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = (&'a str, Entry)>,
    {
        let iter = iter.into_iter();
        let (lower, _) = iter.size_hint();
        let mut codex_builder = Codex::builder_with_capacity(lower);
        for (key, entry) in iter {
            codex_builder = codex_builder.add(key, entry);
        }
        codex_builder.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::App;
    use bevy_asset::{
        io::{
            memory::{Dir, MemoryAssetReader},
            AssetSource, AssetSourceId,
        },
        AssetApp, AssetPlugin, AssetServer, Assets, Handle,
    };
    use bevy_core::TaskPoolPlugin;
    use std::path::Path;

    #[derive(Debug, TypePath, Deserialize)]
    struct MenuItem {
        value: u32,
        text: String,
    }

    impl MenuItem {
        pub fn new(value: u32) -> Self {
            Self {
                value,
                text: String::from("standard text"),
            }
        }
    }

    impl CodexSource for MenuItem {
        const EXTENSION: &'static str = "menu.toml";
    }

    const SPAM: Id<MenuItem> = Id::from_tag("spam");
    const CODEXFILE: &str = "[spam]
value = 13
text = 'some text'

[egg]
value = 37
text = 'some other text'
        ";

    #[test]
    fn id_equals() {
        let id: Id<MenuItem> = Id::from_tag("spam");

        assert_eq!(id, Id::from_tag("spam"));
        assert_ne!(id, Id::from_tag("egg"));
    }

    #[test]
    fn codex_from_iter() {
        let codex: Codex<MenuItem> = Codex::from_iter(vec![
            ("spam", MenuItem::new(10)),
            ("bacon", MenuItem::new(11)),
            ("spam", MenuItem::new(12)),
            ("spam", MenuItem::new(13)),
            ("egg", MenuItem::new(14)),
        ]);

        assert_eq!(codex[&SPAM].value, 13);
    }

    #[test]
    fn codex_build() {
        let codex: Codex<MenuItem> = Codex::builder_with_capacity(0)
            .add("spam", MenuItem::new(10))
            .add("bacon", MenuItem::new(11))
            .build();

        assert_eq!(codex[&SPAM].value, 10);
        assert_eq!(codex[&Id::from_tag("bacon")].value, 11);
    }

    #[test]
    fn codex_loader() {
        let root = Dir::default();
        root.insert_asset_text(Path::new("some.menu.toml"), CODEXFILE);

        let mut app = App::new();
        let memory_asset_reader = MemoryAssetReader { root };
        app.register_asset_source(
            AssetSourceId::Default,
            AssetSource::build().with_reader(move || Box::new(memory_asset_reader.clone())),
        )
        .add_plugins((TaskPoolPlugin::default(), AssetPlugin::default()))
        .init_asset::<Codex<MenuItem>>()
        .register_asset_loader(CodexLoader::<MenuItem>::default());
        let asset_server = app.world.resource::<AssetServer>().clone();

        let handle: Handle<Codex<MenuItem>> = asset_server.load("some.menu.toml");
        let asset_id = handle.id();
        app.world.spawn(handle);
        for _ in 0..100 {
            app.update();
            if let Some(codex) = app
                .world
                .resource::<Assets<Codex<MenuItem>>>()
                .get(asset_id)
            {
                assert_eq!(codex.iter().count(), 2);
                assert_eq!(codex[&SPAM].value, 13);
                assert_eq!(codex[&SPAM].text, String::from("some text"));
                break;
            }
        }
    }
}
