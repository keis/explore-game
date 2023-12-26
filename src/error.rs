use thiserror::Error;

#[derive(Error, Debug)]
pub enum ExplError {
    #[error(transparent)]
    IOError(#[from] std::io::Error),
    #[error(transparent)]
    WFCError(#[from] expl_wfc::WFCError),
    #[error(transparent)]
    QueryEntityError(#[from] bevy::ecs::query::QueryEntityError),
    #[error(transparent)]
    QuerySingleError(#[from] bevy::ecs::query::QuerySingleError),
    #[error(transparent)]
    LoadDirectError(#[from] Box<bevy::asset::LoadDirectError>),
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
