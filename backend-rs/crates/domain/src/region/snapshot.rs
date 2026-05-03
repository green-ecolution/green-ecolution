/// Raw DB-row mapping used exclusively for aggregate rehydration.
#[doc(hidden)]
#[derive(Debug, Clone)]
pub struct RegionSnapshot {
    pub id: i32,
    pub name: String,
}
