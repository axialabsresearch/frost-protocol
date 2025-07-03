use frost_protocol::extensions::{
    CompatibilityChecker,
    ExtensionMetadata,
    ExtensionId,
};
use semver::Version;
use std::collections::HashMap;

fn create_test_metadata(name: &str, version: &str, capabilities: Vec<&str>) -> ExtensionMetadata {
    ExtensionMetadata {
        name: name.to_string(),
        version: version.to_string(),
        description: "Test extension".to_string(),
        dependencies: vec![],
        capabilities: capabilities.into_iter().map(String::from).collect(),
    }
}

#[test]
fn test_version_compatibility() {
    let checker = CompatibilityChecker::new(Version::new(1, 0, 0));
    
    // Test compatible version
    let metadata = create_test_metadata("test", "1.0.0", vec![]);
    assert!(checker.check_version_compatibility(&metadata).is_ok());
    
    // Test incompatible version
    let metadata = create_test_metadata("test", "0.9.0", vec![]);
    assert!(checker.check_version_compatibility(&metadata).is_err());
}

#[test]
fn test_feature_compatibility() {
    let mut checker = CompatibilityChecker::new(Version::new(1, 0, 0));
    checker.add_feature_requirement("feature_a".to_string(), vec!["feature_b".to_string()]);
    
    let extension_a = create_test_metadata("test_a", "1.0.0", vec!["feature_a"]);
    let extension_b = create_test_metadata("test_b", "1.0.0", vec!["feature_b"]);
    let id_b = ExtensionId::new("test_b", "1.0.0");
    
    // Test incompatible features
    let result = checker.check_feature_compatibility(
        &extension_a,
        &[(id_b, extension_b)],
    );
    assert!(result.is_err());
    
    // Test compatible features
    let extension_c = create_test_metadata("test_c", "1.0.0", vec!["feature_c"]);
    let id_c = ExtensionId::new("test_c", "1.0.0");
    let result = checker.check_feature_compatibility(
        &extension_a,
        &[(id_c, extension_c)],
    );
    assert!(result.is_ok());
}

#[test]
fn test_dependency_compatibility() {
    let checker = CompatibilityChecker::new(Version::new(1, 0, 0));
    
    let mut metadata = create_test_metadata("test", "1.0.0", vec![]);
    metadata.dependencies.push(ExtensionId::new("dep", "1.0.0"));
    
    let dep_metadata = create_test_metadata("dep", "1.0.0", vec![]);
    let dep_id = ExtensionId::new("dep", "1.0.0");
    
    // Test satisfied dependency
    let result = checker.check_dependency_compatibility(
        &metadata,
        &[(dep_id.clone(), dep_metadata.clone())],
    );
    assert!(result.is_ok());
    
    // Test missing dependency
    let result = checker.check_dependency_compatibility(
        &metadata,
        &[],
    );
    assert!(result.is_err());
    
    // Test incompatible dependency version
    let old_dep_metadata = create_test_metadata("dep", "0.9.0", vec![]);
    let result = checker.check_dependency_compatibility(
        &metadata,
        &[(dep_id, old_dep_metadata)],
    );
    assert!(result.is_err());
}

#[test]
fn test_full_compatibility_check() {
    let mut checker = CompatibilityChecker::new(Version::new(1, 0, 0));
    checker.add_feature_requirement("feature_a".to_string(), vec!["feature_b".to_string()]);
    
    let mut metadata = create_test_metadata("test", "1.0.0", vec!["feature_a"]);
    metadata.dependencies.push(ExtensionId::new("dep", "1.0.0"));
    
    let dep_metadata = create_test_metadata("dep", "1.0.0", vec![]);
    let dep_id = ExtensionId::new("dep", "1.0.0");
    
    // Test full compatibility check
    let result = checker.check_compatibility(
        &metadata,
        &[(dep_id, dep_metadata)],
    );
    assert!(result.is_ok());
} 