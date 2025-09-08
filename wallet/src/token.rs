use thiserror::Error;

pub const TOKEN_SIZE: usize = 64;

#[derive(Debug, Error)]
pub enum TokenConversionError {
    #[error("Token Strings must have ascii encoding")]
    InvalidStringEncoding,
    #[error("Token Strings must have exactly size {}", TOKEN_SIZE)]
    WrongSizedToken,
}

pub struct Token {
    token: [u8; TOKEN_SIZE],
}

impl TryFrom<String> for Token {
    type Error = TokenConversionError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        if !value.is_ascii() {
           return Err(TokenConversionError::InvalidStringEncoding)
        }
        if value.len() != TOKEN_SIZE {
            return Err(TokenConversionError::WrongSizedToken);
        } 
        let bytes = value.as_bytes();
        let mut array: [u8; TOKEN_SIZE] = [0; TOKEN_SIZE];
        array.copy_from_slice(bytes);
        Ok(Token{
            token: array
        })
    }
}
