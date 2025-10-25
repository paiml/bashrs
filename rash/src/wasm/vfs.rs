//! Virtual Filesystem for WASM Bash Runtime
//!
//! In-memory filesystem for sandboxed bash execution.
//!
//! # Example
//!
//! ```rust
//! use bashrs::wasm::vfs::VirtualFilesystem;
//!
//! let mut vfs = VirtualFilesystem::new();
//! vfs.chdir("/tmp").unwrap();
//! assert_eq!(vfs.getcwd().to_str().unwrap(), "/tmp");
//! ```

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use anyhow::{Result, anyhow};

/// Virtual filesystem node
#[derive(Debug, Clone)]
pub enum VfsNode {
    /// Regular file with content
    File {
        content: Vec<u8>,
        permissions: u32,
    },
    /// Directory with children
    Directory {
        children: HashMap<String, VfsNode>,
        permissions: u32,
    },
    /// Symbolic link
    Symlink {
        target: PathBuf,
    },
}

/// In-memory virtual filesystem
#[derive(Clone)]
pub struct VirtualFilesystem {
    /// Root directory
    root: VfsNode,
    /// Current working directory
    cwd: PathBuf,
}

impl VirtualFilesystem {
    /// Create new virtual filesystem with standard Unix directory structure
    pub fn new() -> Self {
        let mut root_children = HashMap::new();

        // Create standard Unix directories
        root_children.insert(
            "tmp".to_string(),
            VfsNode::Directory {
                children: HashMap::new(),
                permissions: 0o777,
            },
        );
        root_children.insert(
            "home".to_string(),
            VfsNode::Directory {
                children: HashMap::new(),
                permissions: 0o755,
            },
        );

        let root = VfsNode::Directory {
            children: root_children,
            permissions: 0o755,
        };

        Self {
            root,
            cwd: PathBuf::from("/"),
        }
    }

    /// Change current working directory
    pub fn chdir<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let resolved = self.resolve_path(path);

        // Check if path exists and is a directory
        if !self.is_directory(&resolved) {
            return Err(anyhow!("Not a directory: {}", resolved.display()));
        }

        self.cwd = resolved;
        Ok(())
    }

    /// Get current working directory
    pub fn getcwd(&self) -> &Path {
        &self.cwd
    }

    /// Resolve path (handle . and .., relative/absolute)
    fn resolve_path<P: AsRef<Path>>(&self, path: P) -> PathBuf {
        let path = path.as_ref();

        let base = if path.is_absolute() {
            PathBuf::from("/")
        } else {
            self.cwd.clone()
        };

        let mut resolved = PathBuf::new();
        for component in base.components().chain(path.components()) {
            match component {
                std::path::Component::RootDir => resolved.push("/"),
                std::path::Component::CurDir => {}, // Skip "."
                std::path::Component::ParentDir => {
                    resolved.pop(); // Handle ".."
                }
                std::path::Component::Normal(name) => resolved.push(name),
                _ => {}
            }
        }

        if resolved.as_os_str().is_empty() {
            PathBuf::from("/")
        } else {
            resolved
        }
    }

    /// Check if path exists
    pub fn exists<P: AsRef<Path>>(&self, path: P) -> bool {
        let resolved = self.resolve_path(path);
        self.get_node(&resolved).is_some()
    }

    /// Check if path is a directory
    fn is_directory<P: AsRef<Path>>(&self, path: P) -> bool {
        let resolved = self.resolve_path(path);
        matches!(self.get_node(&resolved), Some(VfsNode::Directory { .. }))
    }

    /// Get node at path
    fn get_node(&self, path: &Path) -> Option<&VfsNode> {
        if path == Path::new("/") {
            return Some(&self.root);
        }

        let mut current = &self.root;

        for component in path.components().skip(1) {
            // Skip root
            if let std::path::Component::Normal(name) = component {
                let name_str = name.to_str()?;
                match current {
                    VfsNode::Directory { children, .. } => {
                        current = children.get(name_str)?;
                    }
                    _ => return None,
                }
            }
        }

        Some(current)
    }

    /// Get mutable node at path
    fn get_node_mut(&mut self, path: &Path) -> Option<&mut VfsNode> {
        if path == Path::new("/") {
            return Some(&mut self.root);
        }

        let mut current = &mut self.root;

        for component in path.components().skip(1) {
            // Skip root
            if let std::path::Component::Normal(name) = component {
                let name_str = name.to_str()?.to_string();
                match current {
                    VfsNode::Directory { children, .. } => {
                        current = children.get_mut(&name_str)?;
                    }
                    _ => return None,
                }
            }
        }

        Some(current)
    }

    /// Create directory
    pub fn mkdir<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let resolved = self.resolve_path(path);

        if resolved == Path::new("/") {
            return Err(anyhow!("Cannot create root directory"));
        }

        // Get parent directory
        let parent = resolved.parent()
            .ok_or_else(|| anyhow!("Cannot get parent directory"))?;
        let dir_name = resolved.file_name()
            .ok_or_else(|| anyhow!("Cannot get directory name"))?
            .to_str()
            .ok_or_else(|| anyhow!("Invalid UTF-8 in path"))?;

        // Get parent node
        let parent_node = self.get_node_mut(parent)
            .ok_or_else(|| anyhow!("Parent directory does not exist: {}", parent.display()))?;

        match parent_node {
            VfsNode::Directory { children, .. } => {
                children.insert(
                    dir_name.to_string(),
                    VfsNode::Directory {
                        children: HashMap::new(),
                        permissions: 0o755,
                    },
                );
                Ok(())
            }
            _ => Err(anyhow!("Parent is not a directory")),
        }
    }
}

impl Default for VirtualFilesystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vfs_init() {
        // ARRANGE & ACT
        let vfs = VirtualFilesystem::new();

        // ASSERT
        assert_eq!(vfs.getcwd(), Path::new("/"));
    }

    #[test]
    fn test_vfs_chdir_absolute() {
        // ARRANGE
        let mut vfs = VirtualFilesystem::new();

        // ACT
        vfs.chdir("/tmp").unwrap();

        // ASSERT
        assert_eq!(vfs.getcwd(), Path::new("/tmp"));
    }

    #[test]
    fn test_vfs_chdir_relative() {
        // ARRANGE
        let mut vfs = VirtualFilesystem::new();
        vfs.mkdir("/home/user").unwrap();
        vfs.chdir("/home").unwrap();

        // ACT
        vfs.chdir("user").unwrap();

        // ASSERT
        assert_eq!(vfs.getcwd(), Path::new("/home/user"));
    }

    #[test]
    fn test_vfs_chdir_parent() {
        // ARRANGE
        let mut vfs = VirtualFilesystem::new();
        vfs.mkdir("/home/user").unwrap();
        vfs.chdir("/home/user").unwrap();

        // ACT
        vfs.chdir("..").unwrap();

        // ASSERT
        assert_eq!(vfs.getcwd(), Path::new("/home"));
    }

    #[test]
    fn test_vfs_chdir_current() {
        // ARRANGE
        let mut vfs = VirtualFilesystem::new();
        vfs.chdir("/tmp").unwrap();

        // ACT
        vfs.chdir(".").unwrap();

        // ASSERT
        assert_eq!(vfs.getcwd(), Path::new("/tmp"));
    }

    #[test]
    fn test_vfs_chdir_invalid_path() {
        // ARRANGE
        let mut vfs = VirtualFilesystem::new();

        // ACT
        let result = vfs.chdir("/nonexistent");

        // ASSERT
        assert!(result.is_err());
    }

    #[test]
    fn test_vfs_exists_root() {
        // ARRANGE
        let vfs = VirtualFilesystem::new();

        // ACT & ASSERT
        assert!(vfs.exists("/"));
        assert!(vfs.exists("/tmp"));
        assert!(vfs.exists("/home"));
    }

    #[test]
    fn test_vfs_mkdir() {
        // ARRANGE
        let mut vfs = VirtualFilesystem::new();

        // ACT
        vfs.mkdir("/test").unwrap();

        // ASSERT
        assert!(vfs.exists("/test"));
    }
}
