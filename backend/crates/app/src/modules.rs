use std::collections::BTreeMap;

use domain::module::ModuleKey;
use server::{configuration::ModuleSettings, module::Module};

/// Every module this build ships. Adding a module means one line here.
pub fn registered() -> Vec<Box<dyn Module>> {
    vec![
        Box::new(irrigation_server::IrrigationModule),
        Box::new(waste_server::WasteModule),
    ]
}

/// The activation an organization inherits when no ancestor switched the
/// module: `modules.<key>.default_enabled` from the config, else the module's own.
pub fn activation_defaults(
    modules: &[Box<dyn Module>],
    configured: &BTreeMap<String, ModuleSettings>,
) -> BTreeMap<ModuleKey, bool> {
    modules
        .iter()
        .map(|module| {
            let key = module.key();
            let default = configured
                .get(key.as_str())
                .and_then(|m| m.default_enabled)
                .unwrap_or(module.default_enabled());
            (key, default)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use server::module::{ModuleBuildError, ModuleContext, ModuleRuntime};

    struct Fake {
        key: &'static str,
        default_enabled: bool,
    }

    impl Module for Fake {
        fn key(&self) -> ModuleKey {
            ModuleKey::new(self.key).unwrap()
        }
        fn default_enabled(&self) -> bool {
            self.default_enabled
        }
        fn openapi(&self) -> utoipa::openapi::OpenApi {
            utoipa::openapi::OpenApiBuilder::new().build()
        }
        fn build(&self, _ctx: ModuleContext) -> Result<ModuleRuntime, ModuleBuildError> {
            unreachable!("activation defaults never build a module")
        }
    }

    fn modules() -> Vec<Box<dyn Module>> {
        vec![
            Box::new(Fake {
                key: "on",
                default_enabled: true,
            }),
            Box::new(Fake {
                key: "off",
                default_enabled: false,
            }),
        ]
    }

    fn section(default_enabled: Option<bool>) -> ModuleSettings {
        ModuleSettings {
            default_enabled,
            extra: Default::default(),
        }
    }

    fn resolved(configured: BTreeMap<String, ModuleSettings>) -> Vec<(String, bool)> {
        activation_defaults(&modules(), &configured)
            .into_iter()
            .map(|(key, enabled)| (key.to_string(), enabled))
            .collect()
    }

    #[test]
    fn without_a_config_section_the_module_decides() {
        assert_eq!(
            resolved(BTreeMap::new()),
            [("off".into(), false), ("on".into(), true)]
        );
    }

    #[test]
    fn a_section_without_default_enabled_keeps_the_module_default() {
        let configured = BTreeMap::from([
            ("on".to_string(), section(None)),
            ("off".to_string(), section(None)),
        ]);
        assert_eq!(
            resolved(configured),
            [("off".into(), false), ("on".into(), true)]
        );
    }

    #[test]
    fn the_config_overrides_the_module_default_both_ways() {
        let configured = BTreeMap::from([
            ("on".to_string(), section(Some(false))),
            ("off".to_string(), section(Some(true))),
        ]);
        assert_eq!(
            resolved(configured),
            [("off".into(), true), ("on".into(), false)]
        );
    }
}
