use std::collections::HashMap;
use anyhow::{Result, anyhow};
use semver::Version;

use super::{ExtensionMetadata, ExtensionId};

/// Compatibility checker for protocol extensions
pub struct CompatibilityChecker {
    /// Minimum protocol version required
    min_protocol_version: Version,
    /// Feature compatibility matrix
    feature_matrix: HashMap<String, Vec<String>>,
}

impl CompatibilityChecker {
    /// Create new compatibility checker
    pub fn new(min_protocol_version: Version) -> Self {
        Self {
            min_protocol_version,
            feature_matrix: HashMap::new(),
        }
    }

    /// Add feature compatibility requirements
    pub fn add_feature_requirement(&mut self, feature: String, compatible_with: Vec<String>) {
        self.feature_matrix.insert(feature, compatible_with);
    }

    /// Check if extension is compatible with protocol version
    pub fn check_version_compatibility(&self, extension: &ExtensionMetadata) -> Result<()> {
        let ext_version = Version::parse(&extension.version)
            .map_err(|e| anyhow!("Invalid extension version: {}", e))?;

        if ext_version < self.min_protocol_version {
            return Err(anyhow!(
                "Extension {} requires protocol version >= {}, but {} is available",
                extension.name,
                self.min_protocol_version,
                ext_version
            ));
        }

        Ok(())
    }

    /// Check if extension features are compatible
    pub fn check_feature_compatibility(
        &self,
        extension: &ExtensionMetadata,
        other_extensions: &[(ExtensionId, ExtensionMetadata)],
    ) -> Result<()> {
        for capability in &extension.capabilities {
            if let Some(incompatible) = self.feature_matrix.get(capability) {
                for (_, other) in other_extensions {
                    for other_cap in &other.capabilities {
                        if incompatible.contains(other_cap) {
                            return Err(anyhow!(
                                "Extension {} capability '{}' is incompatible with extension {} capability '{}'",
                                extension.name,
                                capability,
                                other.name,
                                other_cap
                            ));
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
    ) -> Result<()> {
        let available: HashMap<_, _> = available_extensions
            .iter()
            .map(|(id, meta)| (id, meta))
            .collect();

        for dep_id in &extension.dependencies {
            match available.get(dep_id) {
                Some(dep_meta) => {
                    let dep_version = Version::parse(&dep_meta.version)
                        .map_err(|e| anyhow!("Invalid dependency version: {}", e))?;
                    
                    let required_version = Version::parse(
                        dep_id.0
                            .split('@')
                            .nth(1)
                            .ok_or_else(|| anyhow!("Invalid dependency ID format"))?
                    ).map_err(|e| anyhow!("Invalid version requirement: {}", e))?;

                    if dep_version < required_version {
                        return Err(anyhow!(
                            "Extension {} requires {} version >= {}, but {} is available",
                            extension.name,
                            dep_meta.name,
                            required_version,
                            dep_version
                        ));
                    }
                }
                None => {
                    return Err(anyhow!(
                        "Extension {} requires dependency {}, which is not available",
                        extension.name,
                        dep_id.0
                    ));
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
    ) -> Result<()> {
        self.check_version_compatibility(extension)?;
        self.check_feature_compatibility(extension, available_extensions)?;
        self.check_dependency_compatibility(extension, available_extensions)?;
        Ok(())
    }
}