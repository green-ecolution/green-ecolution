//! The core's API description must read the same whichever modules a build
//! ships, so it may not name one.

const ACTIONS: [&str; 4] = ["read", "create", "update", "delete"];

fn is_word_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

fn whole_word_hits(text: &str, word: &str) -> usize {
    text.match_indices(word)
        .filter(|(start, _)| {
            let before = text[..*start].chars().next_back();
            let after = text[start + word.len()..].chars().next();
            !before.is_some_and(is_word_char) && !after.is_some_and(is_word_char)
        })
        .count()
}

/// Every way a text can name a module: its key as a word, one of its
/// permissions, or one of its resources or capacity kinds as a quoted value.
fn module_terms_in(
    text: &str,
    keys: &[String],
    resources: &[String],
    capacity_kinds: &[String],
) -> Vec<String> {
    let mut found = Vec::new();
    for key in keys {
        if whole_word_hits(text, key) > 0 {
            found.push(key.clone());
        }
    }
    for resource in resources {
        for action in ACTIONS {
            let permission = format!("{resource}:{action}");
            if whole_word_hits(text, &permission) > 0 {
                found.push(permission);
            }
        }
        let quoted = format!("\"{resource}\"");
        if text.contains(&quoted) {
            found.push(quoted);
        }
    }
    for kind in capacity_kinds {
        let quoted = format!("\"{kind}\"");
        if text.contains(&quoted) {
            found.push(quoted);
        }
    }
    found
}

#[test]
fn the_check_finds_a_module_key_a_permission_and_a_quoted_resource() {
    let text = r#"module "irrigation", permission tree:read, kind "tree", an organization tree"#;

    let found = module_terms_in(
        text,
        &["irrigation".to_string()],
        &["tree".to_string()],
        &[],
    );

    assert_eq!(found, ["irrigation", "tree:read", "\"tree\""]);
}

#[test]
fn the_check_ignores_a_resource_inside_a_longer_word() {
    let text = "the organization tree, a subtree, street:read";

    let found = module_terms_in(text, &["tree".to_string()], &["tree".to_string()], &[]);

    assert_eq!(found, ["tree"]);
}

#[test]
fn the_check_finds_a_quoted_capacity_kind() {
    let text = r#"{"capacities": {"water": 8000.0}, "note": "water level"}"#;

    let found = module_terms_in(text, &[], &[], &["water".to_string()]);

    assert_eq!(found, ["\"water\""]);
}

#[test]
fn core_openapi_names_no_registered_module() {
    let modules = app::modules::registered();
    let keys: Vec<String> = modules.iter().map(|m| m.key().to_string()).collect();
    let resources: Vec<String> = crate::helpers::registered_resources()
        .iter()
        .map(|resource| resource.as_str().to_string())
        .collect();
    let capacity_kinds: Vec<String> = modules
        .iter()
        .flat_map(|m| m.capacity_kinds())
        .map(|spec| spec.kind.to_string())
        .collect();
    let doc = serde_json::to_string(&server::http::openapi_doc("/", Vec::new())).unwrap();

    let offenders = module_terms_in(&doc, &keys, &resources, &capacity_kinds);

    assert!(offenders.is_empty(), "{offenders:#?}");
}
