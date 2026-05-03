/// Raw DB-row mapping used exclusively for aggregate rehydration.
#[derive(Debug, Clone)]
pub struct RegionSnapshot {
    pub id: i32,
    pub name: String,
}
