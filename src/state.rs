use std::{
    collections::HashMap,
    sync::Arc,
};

use tokio::sync::Mutex;

#[derive(Clone)]
pub struct AppState {
    pub pending_oauth: Arc<Mutex<HashMap<String, String>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            pending_oauth: Arc::new(
                Mutex::new(HashMap::new())
            ),
        }
    }
}