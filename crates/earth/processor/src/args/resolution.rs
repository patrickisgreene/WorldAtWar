use std::str::FromStr;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct ImageResolution {
    pub height: u32,
    pub width: u32
}

impl std::fmt::Display for ImageResolution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}x{}", self.width, self.height)
    }
}

pub enum ImageResolutionError {
    InvalidSeperator,
    Int(std::num::ParseIntError)
}

impl From<std::num::ParseIntError> for ImageResolutionError {
    fn from(value: std::num::ParseIntError) -> Self {
        ImageResolutionError::Int(value)
    }
}

impl FromStr for ImageResolution {
    type Err = ImageResolutionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let chunks: Vec<&str> = s.split("x").collect();

        if chunks.len() != 2 {
            return Err(ImageResolutionError::InvalidSeperator);
        }

        Ok(ImageResolution {
            width: u32::from_str(chunks[0])?,
            height: u32::from_str(chunks[1])?
        })
    }
}

#[derive(Copy, Clone)]
pub struct ImageResolutionParser;

impl clap::builder::TypedValueParser for ImageResolutionParser {
    type Value = ImageResolution;

    fn parse_ref(
        &self,
        cmd: &clap::Command,
        arg: Option<&clap::Arg>,
        value: &std::ffi::OsStr,
    ) -> Result<Self::Value, clap::Error> {
        let inner_parser = clap::builder::StringValueParser::new();
        let s = inner_parser.parse_ref(cmd, arg, value)?;
        ImageResolution::from_str(&s).map_err(|_e| {
            clap::Error::raw(
                clap::error::ErrorKind::ValueValidation,
                format!("Invalid resolution format: '{}'. Expected format: WIDTHxHEIGHT (e.g., 1920x1080)\n", s),
            )
        })
    }

    fn possible_values(
        &self,
    ) -> Option<Box<dyn Iterator<Item = clap::builder::PossibleValue> + '_>> {
        let inner_parser = clap::builder::StringValueParser::new();
        #[allow(clippy::needless_collect)] // Erasing a lifetime
        inner_parser.possible_values().map(|ps| {
            let ps = ps.collect::<Vec<_>>();
            let ps: Box<dyn Iterator<Item = clap::builder::PossibleValue> + '_> =
                Box::new(ps.into_iter());
            ps
        })
    }
}

/// Default to `TargetVersionParser` for `TargetVersion`, instead of `FromStr`
impl clap::builder::ValueParserFactory for ImageResolution {
    type Parser = ImageResolutionParser;

    fn value_parser() -> Self::Parser {
        ImageResolutionParser
    }
}