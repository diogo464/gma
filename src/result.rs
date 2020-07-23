use crate::GMAError;

pub type Result<T, E = GMAError> = std::result::Result<T, E>;
