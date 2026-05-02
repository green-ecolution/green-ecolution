#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "tree_soil_condition", rename_all = "snake_case")]
pub enum SoilCondition {
    Schluffig,
    Sandig,
    Lehmig,
    Tonig,
    Unknown,
}
