use bevy_app::{App, Plugin};
use bevy_asset::io::Reader;
use bevy_asset::{Asset, AssetApp, AssetLoader, LoadContext};
use std::fmt;
use std::error::Error;
use std::marker::PhantomData;
use serde::Deserialize;

/// Per type RON asset loading plugin.
pub struct RonAssetPlugin<A> {
    extension: &'static str,
    _marker: PhantomData<A>,
}

impl<A> Plugin for RonAssetPlugin<A>
where
    for<'de> A: Deserialize<'de> + Asset,
{
    fn build(&self, app: &mut App) {
        app.init_asset::<A>()
            .register_asset_loader(RonAssetLoader::<A> {
                extensions: vec![self.extension],
                _marker: PhantomData,
            });
    }
}

impl<A> RonAssetPlugin<A> where for<'de> A: Deserialize<'de> + Asset {
    /// Create a new plugin that will load assets from files with the given extension.
    pub fn new(extension: &'static str) -> Self {
        Self {
            extension,
            _marker: PhantomData,
        }
    }
}

/// Asset loader for a Serializable RON struct.
pub struct RonAssetLoader<A> {
    extensions: Vec<&'static str>,
    _marker: PhantomData<A>,
}

/// Possible errors that can be produced by [`RonAssetLoader`]
#[derive(Debug)]
pub enum RonLoaderError {
    /// IO Related errors
    Io(std::io::Error),
    /// Serialize / Deserialization related errors.
    Ron(ron::error::SpannedError),
}

impl fmt::Display for RonLoaderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RonLoaderError::Io(io) => write!(f, "{}", io),
            RonLoaderError::Ron(err) => write!(f, "{}", err)
        }
    }
}

impl From<std::io::Error> for RonLoaderError {
    fn from(value: std::io::Error) -> Self {
        RonLoaderError::Io(value)
    }
}

impl From<ron::error::SpannedError> for RonLoaderError {
    fn from(value: ron::error::SpannedError) -> Self {
        RonLoaderError::Ron(value)
    }
}

impl Error for RonLoaderError {}

impl<A> AssetLoader for RonAssetLoader<A> where for<'de> A: Deserialize<'de> + Asset {
    type Asset = A;
    type Settings = ();
    type Error = RonLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let asset = ron::de::from_bytes::<A>(&bytes)?;
        Ok(asset)
    }

    fn extensions(&self) -> &[&str] {
        &self.extensions
    }
}