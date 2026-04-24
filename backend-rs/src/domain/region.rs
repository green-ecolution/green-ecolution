use chrono::{DateTime, Utc};

use crate::domain::Id;

#[derive(Debug, Clone)]
pub struct Region {
    id: Id<Self>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    name: String,
}

impl Region {
    pub fn new(id: Id<Self>, created_at: DateTime<Utc>, updated_at: DateTime<Utc>, name: String) -> Self {
        Self { id, created_at, updated_at, name }
    }

    pub fn id(&self) -> &Id<Self> { &self.id }
    pub fn created_at(&self) -> DateTime<Utc> { self.created_at }
    pub fn updated_at(&self) -> DateTime<Utc> { self.updated_at }
    pub fn name(&self) -> &str { &self.name }
}
