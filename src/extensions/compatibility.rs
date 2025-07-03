use std::collections::HashMap;
use semver::{Version, VersionReq};
use super::{
    ExtensionMetadata,
    ExtensionId,
    errors::{ExtensionError, ExtensionResult},
};

/// Compatibility checker for protocol extensions
pub struct CompatibilityChecker {
    /// Minimum protocol version required
    min_protocol_version: Version,
    /// Version requirements for extensions
    version_requirements: HashMap<String, VersionReq>,
    /// Feature compatibility matrix
    feature_matrix: HashMap<String, Vec<String>>,
}

impl CompatibilityChecker {
    /// Create new compatibility checker
    pub fn new(min_protocol_version: Version) -> Self {
        Self {
            min_protocol_version,
            version_requirements: HashMap::new(),
            feature_matrix: HashMap::new(),
        }
    }

    /// Add version requirement for an extension
    pub fn add_version_requirement(&mut self, extension: String, requirement: VersionReq) {
        self.version_requirements.insert(extension, requirement);
    }

    /// Add feature compatibility requirements
    pub fn add_feature_requirement(&mut self, feature: String, incompatible_with: Vec<String>) {
        self.feature_matrix.insert(feature, incompatible_with);
    }

    /// Check if extension is compatible with protocol version
    pub fn check_version_compatibility(&self, extension: &ExtensionMetadata) -> ExtensionResult<()> {
        let ext_version = Version::parse(&extension.version)
            .map_err(|e| ExtensionError::VersionParseError(e.to_string()))?;

        // Check minimum protocol version
        if ext_version < self.min_protocol_version {
            return Err(ExtensionError::IncompatibleVersion {
                extension: extension.name.clone(),
                required: self.min_protocol_version.to_string(),
                actual: ext_version.to_string(),
            });
        }

        // Check specific version requirements
        if let Some(req) = self.version_requirements.get(&extension.name) {
            if !req.matches(&ext_version) {
                return Err(ExtensionError::IncompatibleVersion {
                    extension: extension.name.clone(),
                    required: req.to_string(),
                    actual: ext_version.to_string(),
                });
            }
        }

        Ok(())
    }

    /// Check if extension features are compatible
    pub fn check_feature_compatibility(
        &self,
        extension: &ExtensionMetadata,
        other_extensions: &[(ExtensionId, ExtensionMetadata)],
    ) -> ExtensionResult<()> {
        for capability in &extension.capabilities {
            if let Some(incompatible) = self.feature_matrix.get(capability) {
                for (_, other) in other_extensions {
                    for other_cap in &other.capabilities {
                        if incompatible.contains(other_cap) {
                            return Err(ExtensionError::DependencyError(format!(
                                "Extension {} capability '{}' is incompatible with extension {} capability '{}'",
                                extension.name,
                                capability,
                                other.name,
                                other_cap
                            )));
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Check if extension dependencies are satisfied
    pub fn check_dependency_compatibility(
        &self,
        extension: &ExtensionMetadata,
        available_extensions: &[(ExtensionId, ExtensionMetadata)],
    ) -> ExtensionResult<()> {
        let available: HashMap<_, _> = available_extensions
            .iter()
            .map(|(id, meta)| (id, meta))
            .collect();

        for dep_id in &extension.dependencies {
            match available.get(dep_id) {
                Some(dep_meta) => {
                    let dep_version = Version::parse(&dep_meta.version)
                        .map_err(|e| ExtensionError::VersionParseError(e.to_string()))?;
                    
                    let required_version = Version::parse(
                        dep_id.0
                            .split('@')
                            .nth(1)
                            .ok_or_else(|| ExtensionError::DependencyError(
                                "Invalid dependency ID format".to_string()
                            ))?
                    ).map_err(|e| ExtensionError::VersionParseError(e.to_string()))?;

                    if dep_version < required_version {
                        return Err(ExtensionError::DependencyError(format!(
                            "Extension {} requires {} version >= {}, but {} is available",
                            extension.name,
                            dep_meta.name,
                            required_version,
                            dep_version
                        )));
                    }
                }
                None => {
                    return Err(ExtensionError::DependencyError(format!(
                        "Extension {} requires dependency {}, which is not available",
                        extension.name,
                        dep_id.0
                    )));
                }
            }
        }

        Ok(())
    }

    /// Perform full compatibility check
    pub fn check_compatibility(
        &self,
        extension: &ExtensionMetadata,
        available_extensions: &[(ExtensionId, ExtensionMetadata)],
    ) -> ExtensionResult<()> {
        self.check_version_compatibility(extension)?;
        self.check_feature_compatibility(extension, available_extensions)?;
        self.check_dependency_compatibility(extension, available_extensions)?;
        Ok(())
    }
}