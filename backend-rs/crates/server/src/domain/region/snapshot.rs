/// Raw DB-row mapping used exclusively for aggregate rehydration.
#[derive(Debug, Clone)]
pub(crate) struct RegionSnapshot {
    pub(crate) id: i32,
    pub(crate) name: String,
}
