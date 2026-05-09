use arch_test_core::access_rules::MayNotAccess;
use arch_test_core::{Architecture, ModuleTree, hash_set};

#[test]
fn core_layers_follow_hexagonal_boundaries() {
    let architecture = Architecture::new(hash_set![
        "domain".to_owned(),
        "application".to_owned(),
        "adapter".to_owned(),
        "input".to_owned(),
        "output".to_owned(),
        "web".to_owned(),
    ])
    .with_access_rule(MayNotAccess::new(
        "domain".to_owned(),
        hash_set!["application".to_owned(), "adapter".to_owned()],
        true,
    ))
    .with_access_rule(MayNotAccess::new(
        "application".to_owned(),
        hash_set!["adapter".to_owned()],
        true,
    ))
    .with_access_rule(MayNotAccess::new(
        "input".to_owned(),
        hash_set!["output".to_owned()],
        true,
    ))
    .with_access_rule(MayNotAccess::new(
        "output".to_owned(),
        hash_set!["input".to_owned()],
        true,
    ));

    let module_tree = ModuleTree::new("src/lib.rs");

    architecture
        .validate_access_rules()
        .unwrap_or_else(|failure| {
            panic!("Invalid architecture rule configuration: {failure:?}");
        });
    architecture
        .check_access_rules(&module_tree)
        .unwrap_or_else(|failure| {
            panic!("Architecture rule violation: {failure:?}");
        });
}
