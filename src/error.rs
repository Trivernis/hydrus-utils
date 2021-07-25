use hydrus_api::error::Error as HydrusError;
use pixiv_rs::error::Error as PixivError;
use rustnao::Error as RustNaoError;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Pixiv(#[from] PixivError),

    #[error("{0}")]
    RustNao(String),

    #[error(transparent)]
    Hydrus(#[from] HydrusError),

    #[error(transparent)]
    Io(#[from] std::io::Error),
}

impl From<RustNaoError> for Error {
    fn from(e: RustNaoError) -> Self {
        Self::RustNao(e.to_string())
    }
}
