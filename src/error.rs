use thiserror::Error;

#[derive(Error, Debug)]
pub enum ExplError {
    #[error(transparent)]
    IOError(#[from] std::io::Error),
    #[error(transparent)]
    WFCError(#[from] expl_wfc::WFCError),
    #[error("could not place portal")]
    CouldNotPlacePortal,
    #[error("could not place spawner")]
    CouldNotPlaceSpawner,
    #[error("could not place party")]
    CouldNotPlaceParty,
}
