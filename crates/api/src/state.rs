use shared::store::Store;

#[derive(Clone)]
pub struct AppState {
    pub store: Store,
}
