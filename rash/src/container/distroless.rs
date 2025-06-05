use crate::models::{Error, Result};

#[derive(Debug, Clone)]
pub enum ContainerFormat {
    #[allow(clippy::upper_case_acronyms)]
    OCI,
    Docker,
}

pub struct DistrolessBuilder {
    scratch: bool,
    static_binary: Vec<u8>,
    format: ContainerFormat,
}

impl DistrolessBuilder {
    pub fn new(binary: Vec<u8>) -> Self {
        Self {
            scratch: true,
            static_binary: binary,
            format: ContainerFormat::OCI,
        }
    }

    pub fn with_format(mut self, format: ContainerFormat) -> Self {
        self.format = format;
        self
    }

    pub fn build(&self) -> Result<Vec<u8>> {
        match self.format {
            ContainerFormat::OCI => self.build_oci(),
            ContainerFormat::Docker => self.build_docker(),
        }
    }

    fn build_oci(&self) -> Result<Vec<u8>> {
        // Create OCI image structure
        let config = self.create_oci_config()?;
        let layer = self.create_binary_layer()?;

        // For now, return a simple tar archive
        self.create_tar_archive(config, layer)
    }

    fn build_docker(&self) -> Result<Vec<u8>> {
        // Create Dockerfile
        let dockerfile = if self.scratch {
            r#"FROM scratch
COPY rash /rash
USER 65534:65534
ENTRYPOINT ["/rash"]
"#
            .to_string()
        } else {
            r#"FROM alpine:3.19
RUN apk add --no-cache dash
COPY rash /usr/local/bin/rash
USER nobody
ENTRYPOINT ["/usr/local/bin/rash"]
"#
            .to_string()
        };

        Ok(dockerfile.into_bytes())
    }

    fn create_oci_config(&self) -> Result<Vec<u8>> {
        let config = serde_json::json!({
            "architecture": "amd64",
            "os": "linux",
            "config": {
                "Entrypoint": ["/rash"],
                "Env": ["PATH=/"],
                "WorkingDir": "/",
                "User": "65534:65534"
            },
            "rootfs": {
                "type": "layers",
                "diff_ids": ["sha256:0000000000000000000000000000000000000000000000000000000000000000"]
            }
        });

        serde_json::to_vec(&config)
            .map_err(|e| Error::Internal(format!("Failed to serialize config: {e}")))
    }

    fn create_binary_layer(&self) -> Result<Vec<u8>> {
        // Compress binary with zstd
        zstd::encode_all(&self.static_binary[..], 19)
            .map_err(|e| Error::Internal(format!("Failed to compress layer: {e}")))
    }

    fn create_tar_archive(&self, config: Vec<u8>, layer: Vec<u8>) -> Result<Vec<u8>> {
        use tar::{Builder, Header};

        let mut ar = Builder::new(Vec::new());

        // Add config.json
        let mut header = Header::new_gnu();
        header.set_path("config.json")?;
        header.set_size(config.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        ar.append(&header, &config[..])?;

        // Add layer
        let mut header = Header::new_gnu();
        header.set_path("layer.tar.zst")?;
        header.set_size(layer.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        ar.append(&header, &layer[..])?;

        // Add manifest.json
        let manifest = serde_json::json!([{
            "Config": "config.json",
            "Layers": ["layer.tar.zst"],
            "RepoTags": ["rash:latest"]
        }]);
        let manifest_bytes = serde_json::to_vec(&manifest)?;

        let mut header = Header::new_gnu();
        header.set_path("manifest.json")?;
        header.set_size(manifest_bytes.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        ar.append(&header, &manifest_bytes[..])?;

        ar.into_inner()
            .map_err(|e| Error::Internal(format!("Failed to create tar: {e}")))
    }
}

/// Generate a minimal Dockerfile for building static binaries
pub fn generate_build_dockerfile() -> String {
    r#"# Build stage for static RASH binary
FROM rust:1.75-alpine AS builder

# Install build dependencies
RUN apk add --no-cache musl-dev

# Copy source
WORKDIR /build
COPY . .

# Build static binary
RUN cargo build --release --target x86_64-unknown-linux-musl

# Runtime stage
FROM scratch
COPY --from=builder /build/target/x86_64-unknown-linux-musl/release/rash /rash
USER 65534:65534
ENTRYPOINT ["/rash"]
"#
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dockerfile_generation() {
        let builder = DistrolessBuilder::new(vec![1, 2, 3]);
        let dockerfile = builder.build_docker().unwrap();
        let content = String::from_utf8(dockerfile).unwrap();

        assert!(content.contains("FROM scratch"));
        assert!(content.contains("USER 65534:65534"));
        assert!(content.contains("ENTRYPOINT"));
    }

    #[test]
    fn test_build_dockerfile() {
        let dockerfile = generate_build_dockerfile();
        assert!(dockerfile.contains("rust:1.75-alpine"));
        assert!(dockerfile.contains("x86_64-unknown-linux-musl"));
    }
}
