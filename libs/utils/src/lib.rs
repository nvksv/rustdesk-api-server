mod tokens;
mod bearer;
mod address_book;
mod types;

pub use tokens::Token;
pub use bearer::BearerToken;
pub use address_book::AddressBook;
pub use types::*;

#[macro_export]
macro_rules! unwrap_or_return {
    ($optval:expr) => {
        match $optval {
            Ok(val) => val,
            Err(err) => {
                tracing::debug!("ERR in unwrap_or_return: {:?}", &err);
                return err;
            },
        }
    };
}

