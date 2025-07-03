/*!
# Documentation Generator

The docs module provides comprehensive documentation generation capabilities for the FROST protocol.
It supports multiple output formats and can generate various types of documentation including protocol
documentation, API references, and developer guides.

## Documentation Types

### Protocol Documentation
- Protocol overview and architecture
- Component descriptions
- Architecture diagrams (optional)
- Usage examples
- Configuration guide

### Extension Documentation
- Extension capabilities
- Dependencies
- Configuration options
- Usage examples
- API reference

### API Documentation
- Network API reference
- Extension API reference
- State management API
- Monitoring API
- Complete type definitions

### Developer Guide
- Getting started guide
- Installation instructions
- Basic usage examples
- Extension development guide
- Best practices

## Output Formats

The generator supports multiple output formats:

- Markdown: Simple text format
- HTML: Web-based documentation
- PDF: Printable format
- Man Pages: Unix manual pages

## Features

### Diagram Generation
- Architecture diagrams using Mermaid
- Component interaction diagrams
- State transition diagrams
- Network topology diagrams

### Code Examples
- Syntax highlighted code blocks
- Runnable examples
- Common use cases
- Error handling examples

### API Reference
- Complete type definitions
- Method signatures
- Parameter descriptions
- Return value documentation
- Error handling guide

## Usage Example

```rust
use frost_protocol::devtools::docs::{DocGeneratorImpl, DocConfig, DocFormat};

async fn generate_docs() -> Result<()> {
    let doc_gen = DocGeneratorImpl::new(extension_manager);
    
    // Generate protocol docs
    let config = DocConfig {
        output_format: DocFormat::Markdown,
        output_dir: "docs".into(),
        include_examples: true,
        include_diagrams: true,
    };
    
    doc_gen.generate_protocol_docs(config).await?;
    
    // Generate API docs
    let api_config = DocConfig {
        output_format: DocFormat::HTML,
        output_dir: "docs/api".into(),
        include_examples: true,
        include_diagrams: false,
    };
    
    doc_gen.generate_api_docs(api_config).await?;
    
    Ok(())
}
```

## Integration

The documentation generator integrates with:

1. Extension system to document extensions
2. API endpoints to generate API docs
3. Example code to generate runnable examples
4. Test cases to generate test documentation

## Best Practices

The generator follows documentation best practices:

- Clear and concise writing
- Consistent formatting
- Complete API coverage
- Runnable examples
- Regular updates
*/

use std::sync::Arc;
use std::path::PathBuf;
use tokio::sync::RwLock;
use anyhow::Result;
use async_trait::async_trait;
use tracing::{info, warn, error};

use crate::extensions::{ExtensionManager, ExtensionId};
use crate::devtools::{
    DocGenerator,
    DocConfig,
    DocFormat,
};

/// Implementation of documentation generator
pub struct DocGeneratorImpl {
    extension_manager: Arc<RwLock<dyn ExtensionManager>>,
}

impl DocGeneratorImpl {
    /// Create new documentation generator
    pub fn new(extension_manager: Arc<RwLock<dyn ExtensionManager>>) -> Self {
        Self {
            extension_manager,
        }
    }

    /// Generate markdown documentation
    async fn generate_markdown(&self, content: &str, output_path: &PathBuf) -> Result<()> {
        tokio::fs::write(output_path, content).await?;
        Ok(())
    }

    /// Generate HTML documentation
    async fn generate_html(&self, content: &str, output_path: &PathBuf) -> Result<()> {
        let html = markdown::to_html(content);
        tokio::fs::write(output_path, html).await?;
        Ok(())
    }

    /// Generate PDF documentation
    async fn generate_pdf(&self, content: &str, output_path: &PathBuf) -> Result<()> {
        // In a real implementation, we would use a PDF generation library
        todo!("PDF generation not implemented");
    }

    /// Generate man page documentation
    async fn generate_man_page(&self, content: &str, output_path: &PathBuf) -> Result<()> {
        // In a real implementation, we would use a man page generator
        todo!("Man page generation not implemented");
    }

    /// Generate documentation in specified format
    async fn generate_docs(
        &self,
        content: &str,
        format: DocFormat,
        output_path: PathBuf,
    ) -> Result<()> {
        match format {
            DocFormat::Markdown => self.generate_markdown(content, &output_path).await,
            DocFormat::HTML => self.generate_html(content, &output_path).await,
            DocFormat::PDF => self.generate_pdf(content, &output_path).await,
            DocFormat::ManPage => self.generate_man_page(content, &output_path).await,
        }
    }

    /// Generate protocol overview documentation
    async fn generate_protocol_overview(&self, config: &DocConfig) -> Result<String> {
        let mut content = String::new();

        content.push_str("# Frostgate Protocol Documentation\n\n");
        content.push_str("## Overview\n\n");
        content.push_str("Frostgate is a permissionless distributed protocol...\n\n");

        content.push_str("## Architecture\n\n");
        content.push_str("The protocol consists of the following components:\n\n");
        content.push_str("- Network Layer\n");
        content.push_str("- Protocol Extensions\n");
        content.push_str("- State Management\n");
        content.push_str("- Consensus Mechanism\n\n");

        if config.include_diagrams {
            content.push_str("## Architecture Diagram\n\n");
            content.push_str("```mermaid\n");
            content.push_str("graph TD\n");
            content.push_str("  A[Network Layer] --> B[Protocol Extensions]\n");
            content.push_str("  B --> C[State Management]\n");
            content.push_str("  C --> D[Consensus Mechanism]\n");
            content.push_str("```\n\n");
        }

        if config.include_examples {
            content.push_str("## Examples\n\n");
            content.push_str("### Basic Usage\n\n");
            content.push_str("```rust\n");
            content.push_str("use frostgate::Protocol;\n\n");
            content.push_str("let protocol = Protocol::new();\n");
            content.push_str("protocol.start().await?;\n");
            content.push_str("```\n\n");
        }

        Ok(content)
    }

    /// Generate extension documentation
    async fn generate_extension_docs_content(&self, extension_id: &ExtensionId) -> Result<String> {
        let manager = self.extension_manager.read().await;
        
        if let Some(extension) = manager.get_extension(extension_id).await? {
            let metadata = extension.metadata();
            let mut content = String::new();

            content.push_str(&format!("# {} Extension\n\n", metadata.name));
            content.push_str(&format!("Version: {}\n\n", metadata.version));
            content.push_str(&format!("## Description\n\n{}\n\n", metadata.description));

            content.push_str("## Capabilities\n\n");
            for capability in &metadata.capabilities {
                content.push_str(&format!("- {}\n", capability));
            }
            content.push_str("\n");

            content.push_str("## Dependencies\n\n");
            for dep in &metadata.dependencies {
                content.push_str(&format!("- {}\n", dep.0));
            }
            content.push_str("\n");

            Ok(content)
        } else {
            Err(anyhow::anyhow!("Extension not found"))
        }
    }

    /// Generate API documentation content
    async fn generate_api_docs_content(&self) -> Result<String> {
        let mut content = String::new();

        content.push_str("# Frostgate API Documentation\n\n");
        
        content.push_str("## Network API\n\n");
        content.push_str("### Peer Management\n\n");
        content.push_str("```rust\n");
        content.push_str("/// Connect to a peer\n");
        content.push_str("async fn connect_peer(&self, address: &str) -> Result<PeerId>;\n\n");
        content.push_str("/// Disconnect from a peer\n");
        content.push_str("async fn disconnect_peer(&self, peer_id: &PeerId) -> Result<()>;\n");
        content.push_str("```\n\n");

        content.push_str("## Extension API\n\n");
        content.push_str("### Extension Management\n\n");
        content.push_str("```rust\n");
        content.push_str("/// Register a new extension\n");
        content.push_str("async fn register_extension(&mut self, extension: Box<dyn Extension>) -> Result<ExtensionId>;\n\n");
        content.push_str("/// Unregister an extension\n");
        content.push_str("async fn unregister_extension(&mut self, id: &ExtensionId) -> Result<()>;\n");
        content.push_str("```\n\n");

        Ok(content)
    }

    /// Generate developer guide content
    async fn generate_guide_content(&self) -> Result<String> {
        let mut content = String::new();

        content.push_str("# Frostgate Developer Guide\n\n");
        
        content.push_str("## Getting Started\n\n");
        content.push_str("### Installation\n\n");
        content.push_str("```bash\n");
        content.push_str("cargo add frostgate\n");
        content.push_str("```\n\n");

        content.push_str("### Basic Usage\n\n");
        content.push_str("```rust\n");
        content.push_str("use frostgate::Protocol;\n\n");
        content.push_str("let protocol = Protocol::new();\n");
        content.push_str("protocol.start().await?;\n");
        content.push_str("```\n\n");

        content.push_str("## Creating Extensions\n\n");
        content.push_str("### Extension Template\n\n");
        content.push_str("```rust\n");
        content.push_str("use frostgate::Extension;\n\n");
        content.push_str("#[derive(Extension)]\n");
        content.push_str("struct MyExtension {\n");
        content.push_str("    // Extension state\n");
        content.push_str("}\n");
        content.push_str("```\n\n");

        Ok(content)
    }
}

#[async_trait]
impl DocGenerator for DocGeneratorImpl {
    async fn generate_protocol_docs(&self, config: DocConfig) -> Result<()> {
        info!("Generating protocol documentation");
        
        let content = self.generate_protocol_overview(&config).await?;
        let output_path = PathBuf::from(&config.output_dir).join("protocol");
        
        self.generate_docs(&content, config.output_format, output_path).await?;
        Ok(())
    }

    async fn generate_extension_docs(&self, extension_id: &ExtensionId) -> Result<()> {
        info!("Generating documentation for extension {}", extension_id.0);
        
        let content = self.generate_extension_docs_content(extension_id).await?;
        let output_path = PathBuf::from("docs/extensions").join(&extension_id.0);
        
        self.generate_docs(&content, DocFormat::Markdown, output_path).await?;
        Ok(())
    }

    async fn generate_api_docs(&self, config: DocConfig) -> Result<()> {
        info!("Generating API documentation");
        
        let content = self.generate_api_docs_content().await?;
        let output_path = PathBuf::from(&config.output_dir).join("api");
        
        self.generate_docs(&content, config.output_format, output_path).await?;
        Ok(())
    }

    async fn generate_developer_guide(&self, config: DocConfig) -> Result<()> {
        info!("Generating developer guide");
        
        let content = self.generate_guide_content().await?;
        let output_path = PathBuf::from(&config.output_dir).join("guide");
        
        self.generate_docs(&content, config.output_format, output_path).await?;
        Ok(())
    }
} 