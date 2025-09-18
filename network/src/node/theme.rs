use std::fmt;
use thiserror::Error;

/// Total types of themes.
pub const N_THEMES: usize = 2;

/// Error type for unknown theme protocol number.
#[derive(Error, Debug)]
pub enum ThemeError {
    /// Uknonwn theme protocol number.
    NoSuchTheme {
        /// Uknonwn theme protocol number.
        n: usize,
    },
}

impl fmt::Display for ThemeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ThemeError::NoSuchTheme { n } => write!(
                f,
                "No theme whose protocol is: {n}. There are {N_THEMES} themes",
            ),
        }
    }
}

/// `[Theme]` expresses the possible gossip message types.
#[derive(Clone, Copy, Default)]
pub enum Theme {
    #[default]
    /// The message will share new `[Neighbour]`s.
    NewNeighbours,
    /// The message will share a `[Chain]` version
    Chain,
}

impl Theme {
    /// Updates the `[Theme]` cyclically.
    pub fn next(&mut self) {
        *self = match *self {
            Theme::Chain => Theme::NewNeighbours,
            Theme::NewNeighbours => Theme::Chain,
        }
    }

    /// Converts `[Theme]` variants to their protocol equivalents.
    #[must_use]
    pub fn to_protocol(&self) -> usize {
        match self {
            Theme::Chain => 0,
            Theme::NewNeighbours => 1,
        }
    }

    /// Converts protocol numbers to their `[Theme]` equivalents.
    pub fn from_protocol(n: usize) -> Result<Self, ThemeError> {
        match n {
            0 => Ok(Theme::Chain),
            1 => Ok(Theme::NewNeighbours),
            _ => Err(ThemeError::NoSuchTheme { n }),
        }
    }
}
