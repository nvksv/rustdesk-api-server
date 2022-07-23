use rocket::{
    Rocket, Build, get, post, routes, State, uri, 
    form::{Form, FromForm},
    http::{Cookie, CookieJar, hyper::header::AUTHORIZATION},
    response::Redirect,
    response::status,
};
use askama::Template;

use utils::CookieAuthToken;
use state::{
    ApiState, UserPasswordInfo,
    ui::UserInfo
};

type AuthenticatedAdmin = state::AuthenticatedAdmin<CookieAuthToken>;

pub fn update_rocket(rocket: Rocket<Build>) -> Rocket<Build> {
    rocket
    .mount("/admin", routes![
        index_unauthorized,
        index,
        login,
        logout
    ])
}


#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    pub current_user: String,
    pub users: Vec<UserInfo>,
}

#[get("/", rank = 1)]
async fn index(
    state: &State<ApiState>,
    user: AuthenticatedAdmin,
) -> IndexTemplate {

    let users = state.ui_get_all_users().await.unwrap_or_else(|| vec![]);

    IndexTemplate { 
        current_user: user.username.clone(),
        users,
    }
}

#[derive(Template)]
#[template(path = "login_form.html")]
struct LoginFormTemplate {}

#[get("/", rank = 2)]
async fn index_unauthorized(
) -> LoginFormTemplate {
    LoginFormTemplate {}
}

#[derive(FromForm)]
struct LoginUserInput {
    username: String,
    password: String,
}

#[post("/login", data = "<user_input>")]
async fn login(
    state: &State<ApiState>,
    cookies: &CookieJar<'_>,
    user_input: Form<LoginUserInput>,
) -> Result<Redirect, status::Forbidden<()>> {
    let status_forbidden = || status::Forbidden::<()>(None);
    let user_password_info = UserPasswordInfo::from_password( user_input.password.as_str() );
    let (_user, access_token) = state.user_login(&user_input.username, user_password_info, false).await.ok_or_else(status_forbidden)?;

    cookies.add(Cookie::build(AUTHORIZATION.as_str(), access_token.to_base64()).secure(true).http_only(true).path("/admin").finish());
    Ok(Redirect::to(uri!("/admin")))
}


#[get("/logout")]
async fn logout(
    state: &State<ApiState>,
    cookies: &CookieJar<'_>,
    user: AuthenticatedAdmin,
) -> Redirect {
    state.user_logout(&user.info).await;

    cookies.remove(Cookie::named(AUTHORIZATION.as_str()));
    Redirect::to(uri!("/admin"))
}
