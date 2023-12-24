use axum::Router;

use super::GlobalState;

mod account;
mod announcement;
mod automate;
mod calendar;
mod certificate;
mod challenge;
mod game;
mod instance;
mod media;
mod platform;
mod user;
mod wiki;

pub fn router(state: &GlobalState) -> Router<GlobalState> {
    Router::new()
        .nest("/account", account::router(state))
        .nest("/announcement", announcement::router(state))
        .nest("/automate", automate::router(state))
        .nest("/certificate", certificate::router(state))
        .nest("/game", game::router(state))
        .nest("/challenge", challenge::router(state))
        .nest("/media", media::router(state))
        .nest("/platform", platform::router(state))
        .nest("/user", user::router(state))
        .nest("/calendar", calendar::router(state))
        .nest("/wiki", wiki::router(state))
        .nest("/instance", instance::router(state))
}
