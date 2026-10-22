pub mod catalog;

#[cfg(test)]
pub use catalog::StaticRecipes;
pub use catalog::{Catalog, CatalogError, JustRecipes, Param, ParamKind, Recipe, RecipeSource};
