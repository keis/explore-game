use crate::ExplError;
pub use bevy::{prelude::*, reflect::TypePath};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
    marker::PhantomData,
};

#[derive(Reflect, Clone, Copy, Eq, PartialEq, Debug)]
pub struct Item;

#[derive(Reflect, Serialize, Deserialize, Clone, Copy, Debug)]
#[reflect(Hash, PartialEq, Serialize, Deserialize)]
pub struct Id<T> {
    value: u64,

    #[reflect(ignore)]
    #[serde(skip)]
    _phantom: PhantomData<T>,
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

    pub const fn from_tag(tag: &str) -> Self {
        Self::new(Self::hash_bytes(tag.as_bytes()))
    }

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

impl<T> Eq for Id<T> {}

impl<T> PartialEq for Id<T> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl<T> Hash for Id<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}

#[derive(Component, Reflect, Clone, Debug, Default)]
#[reflect(Component)]
pub struct Inventory {
    slots: HashMap<Id<Item>, u32>,
}

impl Inventory {
    pub const CRYSTAL: Id<Item> = Id::from_tag("crystal");
    pub const SUPPLY: Id<Item> = Id::from_tag("supply");

    pub fn has_item(&self, item_id: Id<Item>) -> bool {
        self.slots
            .get(&item_id)
            .filter(|&&count| count > 0)
            .is_some()
    }

    pub fn count_item(&self, item_id: Id<Item>) -> u32 {
        self.slots.get(&item_id).copied().unwrap_or(0)
    }

    pub fn add_item(&mut self, item_id: Id<Item>, item_count: u32) {
        self.slots
            .entry(item_id)
            .and_modify(|count| *count += item_count)
            .or_insert(item_count);
    }

    pub fn take_item(&mut self, item_id: Id<Item>, item_request: u32) -> Result<u32, ExplError> {
        let count = self
            .slots
            .get_mut(&item_id)
            .ok_or(ExplError::InventoryItemNotFound)?;
        if *count < item_request {
            return Err(ExplError::InventoryItemNotFound);
        }
        *count -= item_request;
        Ok(item_request)
    }

    pub fn split_item(&mut self, item_id: Id<Item>) -> u32 {
        if let Some(count) = self.slots.get_mut(&item_id) {
            let half = *count / 2;
            *count -= half;
            return half;
        }
        0
    }

    pub fn take_all(&mut self, other: &mut Inventory) {
        for (item_id, item_count) in other.slots.iter_mut() {
            self.slots
                .entry(*item_id)
                .and_modify(|count| *count += *item_count)
                .or_insert(*item_count);
            *item_count = 0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Id, Inventory, Item};
    use crate::ExplError;

    const SPAM: Id<Item> = Id::from_tag("spam");
    const EGG: Id<Item> = Id::from_tag("egg");

    #[test]
    fn add_count_has() {
        let mut inventory = Inventory::default();

        assert!(!inventory.has_item(SPAM));
        assert_eq!(inventory.count_item(SPAM), 0);

        inventory.add_item(SPAM, 1);
        assert!(inventory.has_item(SPAM));
        assert_eq!(inventory.count_item(SPAM), 1);

        inventory.add_item(SPAM, 10);
        assert!(inventory.has_item(SPAM));
        assert_eq!(inventory.count_item(SPAM), 11);

        assert!(!inventory.has_item(EGG));
        assert_eq!(inventory.count_item(EGG), 0);
    }

    #[test]
    fn take() -> Result<(), ExplError> {
        let mut inventory = Inventory::default();
        inventory.add_item(SPAM, 13);
        inventory.add_item(EGG, 7);

        assert_eq!(inventory.take_item(SPAM, 8)?, 8);
        assert_eq!(inventory.count_item(SPAM), 5);
        let result = inventory.take_item(EGG, 8);
        assert!(result.is_err());
        let result = inventory.take_item(Id::from_tag("bacon"), 2);
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn split() {
        let mut inventory = Inventory::default();
        inventory.add_item(SPAM, 13);

        assert_eq!(inventory.split_item(SPAM), 6);
        assert_eq!(inventory.count_item(SPAM), 7);
        assert_eq!(inventory.split_item(EGG), 0);
    }

    #[test]
    fn take_all() {
        let mut source = Inventory::default();
        source.add_item(SPAM, 8);

        let mut target = Inventory::default();
        target.add_item(SPAM, 5);
        target.add_item(EGG, 7);

        target.take_all(&mut source);

        assert_eq!(target.count_item(SPAM), 13);
        assert_eq!(target.count_item(EGG), 7);
        assert_eq!(source.count_item(SPAM), 0);
        assert_eq!(source.count_item(EGG), 0);
    }
}
