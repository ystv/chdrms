use axum::extract::FromRef;

#[derive(Clone, FromRef)]
pub struct AppState {

}

impl AppState {
    pub fn new() -> Self {
        Self {
            
        }
    }
}
