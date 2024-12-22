use bevy::ecs::{entity::Entity, query::QueryEntityError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ExplError {
    #[error(transparent)]
    IOError(#[from] std::io::Error),
    #[error(transparent)]
    WFCError(#[from] expl_wfc::WFCError),
    #[error("query does not match `{0}`")]
    QueryDoesNotMatch(Entity),
    #[error("no such entity `{0}`")]
    NoSuchEntity(Entity),
    #[error("aliased mutability `{0}`")]
    AliasedMutability(Entity),
    #[error(transparent)]
    QuerySingleError(#[from] bevy::ecs::query::QuerySingleError),
    #[error(transparent)]
    LoadDirectError(#[from] Box<bevy::asset::LoadDirectError>),
    #[error("registered system error")]
    RegisteredSystemError,
    #[error("resource missing")]
    ResourceMissing,
    #[error("could not place portal")]
    CouldNotPlacePortal,
    #[error("could not place spawner")]
    CouldNotPlaceSpawner,
    #[error("could not place party")]
    CouldNotPlaceParty,
    #[error("unknown terrain character")]
    UnknownTerrainCharacter,
    #[error("coordinate out of bounds")]
    OutOfBounds,
    #[error("tried to move without movement points")]
    MoveWithoutMovementPoints,
    #[error("invalid location: {0}")]
    InvalidLocation(String),
    #[error("invalid split")]
    InvalidPartySplit,
    #[error("not enough supplies")]
    MissingSupplies,
    #[error("inventory item not found")]
    InventoryItemNotFound,
    #[error("missing codex")]
    MissingCodex,
    #[error("missing template")]
    MissingTemplate,
    #[error("invalid action target")]
    InvalidTarget,
}

impl<I, O> From<bevy::ecs::system::RegisteredSystemError<I, O>> for ExplError
where
    I: bevy::ecs::system::SystemInput,
{
    fn from(_err: bevy::ecs::system::RegisteredSystemError<I, O>) -> Self {
        Self::RegisteredSystemError
    }
}

impl<'a> From<bevy::ecs::query::QueryEntityError<'a>> for ExplError {
    fn from(err: bevy::ecs::query::QueryEntityError<'a>) -> Self {
        match err {
            QueryEntityError::QueryDoesNotMatch(e, _) => Self::QueryDoesNotMatch(e),
            QueryEntityError::NoSuchEntity(e) => Self::NoSuchEntity(e),
            QueryEntityError::AliasedMutability(e) => Self::AliasedMutability(e),
        }
    }
}
