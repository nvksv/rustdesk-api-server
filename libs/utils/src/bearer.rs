use rocket::{
    http::{
        hyper::header::AUTHORIZATION
    },
    request::{Request, FromRequest, Outcome},
};
use crate::{
    unwrap_or_return,
    tokens::Token,
};

#[derive(Debug)]
pub struct BearerToken {
    pub token: Token,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for BearerToken {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let auth_string = unwrap_or_return!(
            request
            .headers()
            .get_one(AUTHORIZATION.as_str())
            .ok_or(Outcome::Forward(()))
        );

        let (bearer, mut token_str) = unwrap_or_return!(
            auth_string
            .split_once(' ')
            .ok_or(Outcome::Forward(()))
        );

        if !bearer.eq_ignore_ascii_case("Bearer") {
            return Outcome::Forward(());
        };

        token_str = token_str.trim();

        let token = unwrap_or_return!(
            Token::from_str(token_str)
            .map_err(|_| Outcome::Forward(()))
        );

        let bearer = Self { token };

        Outcome::Success(bearer)
    }
}

