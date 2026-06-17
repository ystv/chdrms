use utoipa_axum::{router::OpenApiRouter, routes};

use crate::state::AppState;

mod manage;
mod members;
mod permissions;

pub(super) const TAG: &str = "group";

pub fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(manage::list, manage::create))
        .routes(routes!(manage::get))
        .routes(routes!(members::list, members::add, members::remove))
        .routes(routes!(
            permissions::list,
            permissions::add,
            permissions::remove
        ))
}
