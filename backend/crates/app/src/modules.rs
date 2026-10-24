use server::module::Module;

/// Every module this build ships. Adding a module means one line here.
pub fn registered() -> Vec<Box<dyn Module>> {
    Vec::new()
}
