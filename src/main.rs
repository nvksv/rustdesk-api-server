mod api;

use utils;
use state;
#[cfg(feature = "ui")]
use ui;

use rocket::{
    self, routes, post, Build, State, Rocket,
    data::{Limits, ToByteUnit},
    serde::{json::Json},
    response::status,
    config::LogLevel, 
};
use utils::{AddressBook, unwrap_or_return};
use state::{ApiState, AuthenticatedUser, UserPasswordInfo};

use tracing_subscriber;

use crate::{
    api::{LoginRequest, LoginReply, AbGetResponse, AbRequest, AuditRequest, CurrentUserRequest, CurrentUserResponse, UserInfo, LogoutReply},
};

async fn build_rocket() -> Rocket<Build> {
    tracing_subscriber::fmt::init();

    let figment = rocket::Config::figment()
        .merge(("address", "0.0.0.0"))
        .merge(("port", 21114))
        .merge(("log_level", LogLevel::Debug))
        // .merge(("tls.certs", "rustdesk.crt"))
        // .merge(("tls.key", "rustdesk.pem"))
        .merge(("limits", Limits::new().limit("json", 2.mebibytes())));

    let state = ApiState::new_with_db( ".api.db" ).await;

    let mut rocket = rocket::custom(figment)
        .mount("/api", routes![
            login, 
            ab_get, 
            ab, 
            current_user,
            audit, 
            logout
        ])
        .manage( state );

    #[cfg(feature = "ui")]
    {
        rocket = ui::update_rocket(rocket);
    }

    rocket
}

#[rocket::launch]
async fn launch_rocket() -> _{
    build_rocket().await
}


#[post("/login", format = "application/json", data = "<request>")]
async fn login(
    state: &State<ApiState>,
    request: Json<LoginRequest>,
) -> Result<Json<LoginReply>, status::Forbidden<()>> {
    let status_forbidden = || status::Forbidden::<()>(None);

    let user_password_info = UserPasswordInfo::from_password( request.password.as_str() );
    let (user, access_token) = state.user_login(&request.username, user_password_info).await.ok_or_else(status_forbidden)?;

    let reply = LoginReply {
        user: UserInfo { 
            name: user 
        },
        access_token,
    };

    tracing::debug!("login: {:?}", request);

    state.check_maintenance().await;

    Ok(Json(reply))
}

#[post("/ab/get", format = "application/json")]
async fn ab_get(
    state: &State<ApiState>,
    user: AuthenticatedUser,
) -> Result<Json<AbGetResponse>, status::Forbidden<()>> {
    tracing::debug!("ab get");

    let abi = state
        .get_user_address_book(user.user_id)
        .await
        .unwrap_or_else(|| AddressBook::empty());

    let reply = AbGetResponse {
        error: false,
        updated_at: "now".to_string(),
        data: abi.ab
    };

    state.check_maintenance().await;

    tracing::debug!("ab get reply: {:?}", Json(&reply));
    Ok(Json(reply))
}

#[post("/ab", format = "application/json", data = "<request>")]
async fn ab(
    state: &State<ApiState>,
    user: AuthenticatedUser,
    request: Json<AbRequest>,
) -> Result<(), status::Forbidden<()>> {
    tracing::debug!("ab: {:?}", request);

    let ab = request.data.clone();

    tracing::debug!("new ab: {:?}", &ab);

    let ab = AddressBook {
        ab
    };

    let _ = unwrap_or_return!(
        state
        .set_user_address_book(user.user_id, ab)
        .await
        .ok_or(Err(status::Forbidden::<()>(None)))
    );

    state.check_maintenance().await;

    Ok(())
}

#[post("/currentUser", format = "application/json", data = "<request>")]
async fn current_user(
    state: &State<ApiState>,
    user: AuthenticatedUser,
    request: Json<CurrentUserRequest>,
) -> Result<Json<CurrentUserResponse>, status::Forbidden<()>> {
    tracing::debug!("current_user authenticated request: {:?}", request);

    let username = unwrap_or_return!(
        state
        .get_current_user_name(&user)
        .await
        .ok_or(Err(status::Forbidden::<()>(None)))
    );

    let reply = CurrentUserResponse {
        error: false,
        data: UserInfo { 
            name: username 
        }
    };

    tracing::debug!("current_user reply: {:?}", reply);
    Ok(Json(reply))
}

#[post("/audit", format = "application/json", data = "<request>")]
async fn audit(
    state: &State<ApiState>,
    request: Json<AuditRequest>,
) {
    tracing::debug!("audit: {:?}", request);
    state.check_maintenance().await;
}

#[post("/logout", format = "application/json", data = "<request>")]
async fn logout(
    state: &State<ApiState>,
    user: AuthenticatedUser,
    request: Json<CurrentUserRequest>,
) -> Result<Json<LogoutReply>, status::Forbidden<()>> {
    tracing::debug!("logout: {:?}", request);

    let _ = unwrap_or_return!(
        state
        .user_logout(&user)
        .await
        .ok_or(Err(status::Forbidden::<()>(None)))
    );

    let reply = LogoutReply {
        data: String::new()
    };

    state.check_maintenance().await;

    Ok(Json(reply))
}

