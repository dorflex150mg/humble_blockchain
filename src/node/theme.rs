pub mod theme {
    
    use std::fmt;
    use thiserror::Error;

    pub const N_THEMES: usize = 2;

    #[derive(Error, Debug)]
    pub enum ThemeError {
        NoSuchTheme{n: usize}
    }

    impl fmt::Display for ThemeError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                ThemeError::NoSuchTheme{n} => write!(f, "No theme whose protocol is: {}. There are {} themes", n, N_THEMES),
            }
        }
    }

    #[derive(Clone, Copy)]
    pub enum Theme {
        Chain,
        NewNeighbours,
    }

    impl Theme {

        pub fn next(&mut self) {
            *self = match *self {
                Theme::Chain => Theme::NewNeighbours,
                Theme::NewNeighbours => Theme::NewNeighbours,
            }
        }

            
        
        pub fn to_protocol(&self) -> usize {
            match self {
                Theme::Chain => 0,
                Theme::NewNeighbours => 1,
            }
        }


        pub fn from_protocol(n: usize) -> Result<Self, ThemeError> {
            match n {
                0 => Ok(Theme::Chain),
                1 => Ok(Theme::NewNeighbours),
                _ => Err(ThemeError::NoSuchTheme{n}),
            }
        }

    }
}
