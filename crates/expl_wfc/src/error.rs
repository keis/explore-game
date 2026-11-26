use thiserror::Error;

#[derive(Error, Debug)]
pub enum WFCError {
    #[error("incomptabile seed")]
    IncompatibleSeed,
    #[error("invalid seed")]
    InvalidSeed,
    #[error("cell not collapsed")]
    CellNotCollapsed,
    #[error(transparent)]
    IOError(#[from] std::io::Error),
    #[error("cell could not be parsed")]
    CellParseError,
    #[error("collapse budget of {0} exceeded")]
    BudgetExceeded(usize),
    #[error("no history left to rewind")]
    HistoryExhausted,
    #[error("unknown error")]
    UnknownError,
}
