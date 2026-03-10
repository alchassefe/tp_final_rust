use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

pub struct Entry {
    pub value: String,
    pub expires_at: Option<Instant>,
}

impl Entry {
    // Méthode pour vérifier si la clé est expirée
    pub fn is_expired(&self) -> bool {
        self.expires_at.is_some_and(|at| at <= Instant::now())
    }
}
// Créer le store partagé (Arc<Mutex<HashMap<String, ...>>>)
pub type Store = Arc<Mutex<HashMap<String, Entry>>>;
