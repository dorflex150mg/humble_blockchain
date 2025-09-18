use std::ops::Deref;
use thiserror::Error;

pub const TOKEN_SIZE: usize = 64;

#[derive(Debug, Error)]
pub enum TokenConversionError {
    #[error("Token Strings must have ascii encoding")]
    InvalidStringEncoding,
    #[error("Token Strings must have exactly size {}", TOKEN_SIZE)]
    WrongSizedToken(usize),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token([u8; TOKEN_SIZE]);

impl Token {
    pub fn new(array: [u8; TOKEN_SIZE]) -> Self {
        Token(array)
    }
}

impl TryFrom<Token> for String {
    type Error = TokenConversionError;
    fn try_from(value: Token) -> Result<Self, Self::Error> {
        Ok(str::from_utf8((*value).as_slice())
            .map_err(|_| TokenConversionError::InvalidStringEncoding)?
            .to_owned())
    }
}

impl TryFrom<String> for Token {
    type Error = TokenConversionError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        if !value.is_ascii() {
            return Err(TokenConversionError::InvalidStringEncoding);
        }
        if value.len() != TOKEN_SIZE {
            return Err(TokenConversionError::WrongSizedToken(value.len()));
        }
        let bytes = value.as_bytes();
        let mut array: [u8; TOKEN_SIZE] = [0; TOKEN_SIZE];
        array.copy_from_slice(bytes);
        Ok(Token(array))
    }
}

impl Deref for Token {
    type Target = [u8; TOKEN_SIZE];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
