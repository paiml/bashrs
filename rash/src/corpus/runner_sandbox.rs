//! Sandboxes corpus entry execution (PMAT-256 / #318).
//!
//! `bashrs corpus run` transpiles a corpus entry and then *executes* the
//! transpiled shell output to check behavioral equivalence and cross-shell
//! agreement (see `runner_checks.rs`). Before this module existed, that
//! execution inherited the caller's real working directory, `$HOME` and
//! `PATH` wholesale -- so a corpus entry (or an adversarial one) that wrote
//! files was free to write them anywhere the caller could. Measured
//! consequence in this repository: a corpus run left an untracked
//! byte-identical copy of `rash/src/` under `rash/dest/` (1,759 files) plus
//! `job.timer`, `cron.lock`, `backup.tar` and `done` under `rash/`.
//!
//! [`Sandbox`] gives every executed entry a private, disposable working
//! directory that also serves as `$HOME`, plus a minimal `PATH` containing
//! only the directories that hold the interpreters the runner actually
//! needs. On Linux, when `bwrap` (bubblewrap) is available, the command is
//! additionally run inside a `bwrap` sandbox with the network unshared.
//!
//! The sandbox is created once per process (i.e. once per `corpus run`
//! invocation, not once per entry -- creating a fresh tempdir per entry
//! would be far slower across a 17k+ entry corpus) via [`shared`], and is
//! shared read-only across the runner's worker threads.

use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::OnceLock;

/// Interpreters whose directories make up the sandboxed `PATH`.
const REQUIRED_TOOLS: &[&str] = &["sh", "dash", "shellcheck", "make"];

/// Fallback `PATH` used only if none of `REQUIRED_TOOLS` resolve at all.
const FALLBACK_PATH: &str = "/usr/bin:/bin";

/// A once-per-run execution sandbox: a private directory used as both the
/// child's working directory and `$HOME`, a minimal `PATH`, and (when
/// available) a bubblewrap wrapper.
pub(crate) struct Sandbox {
    dir: PathBuf,
    path_env: String,
    bwrap: Option<PathBuf>,
    timeout_bin: PathBuf,
    warn_count: AtomicU32,
}

impl Sandbox {
    /// Build a sandbox, auto-detecting bubblewrap and `timeout` on `PATH`.
    fn new() -> Self {
        Self::assemble(which("bwrap"))
    }

    /// Build a sandbox that never uses bubblewrap, even if installed --
    /// exercises the "bwrap absent" fallback deterministically.
    #[cfg(test)]
    pub(crate) fn new_without_bwrap() -> Self {
        Self::assemble(None)
    }

    /// Build a sandbox from explicit parts, bypassing host auto-detection --
    /// used to deterministically force conditions such as "dash absent"
    /// without relying on (or mutating) this machine's real `PATH`.
    #[cfg(test)]
    pub(crate) fn with_parts_for_test(path_env: String, bwrap: Option<PathBuf>) -> Self {
        Self {
            dir: Self::fresh_tempdir(),
            path_env,
            bwrap,
            timeout_bin: which("timeout").unwrap_or_else(|| PathBuf::from("timeout")),
            warn_count: AtomicU32::new(0),
        }
    }

    fn assemble(bwrap: Option<PathBuf>) -> Self {
        Self {
            dir: Self::fresh_tempdir(),
            path_env: minimal_path(),
            bwrap,
            timeout_bin: which("timeout").unwrap_or_else(|| PathBuf::from("timeout")),
            warn_count: AtomicU32::new(0),
        }
    }

    fn fresh_tempdir() -> PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let dir = std::env::temp_dir().join(format!(
            "bashrs-corpus-sandbox-{}-{nanos}",
            std::process::id()
        ));
        let _ = std::fs::create_dir_all(&dir);
        dir
    }

    /// The sandbox's private directory (used as both cwd and `$HOME`).
    /// Only test code needs to inspect this directly; production call sites
    /// go through [`Sandbox::run`].
    #[cfg(test)]
    pub(crate) fn dir(&self) -> &Path {
        &self.dir
    }

    /// Run `timeout <secs> <interpreter> -c <script>` inside the sandbox.
    /// Preserves the caller-visible contract of the plain
    /// `Command::new("timeout")...output()` call it replaces: stdin is
    /// null, stdout/stderr are discarded, and a 124 exit code means timeout.
    pub(crate) fn run(&self, secs: &str, interpreter: &str, script: &str) -> io::Result<Output> {
        match &self.bwrap {
            Some(bwrap) => self.run_bwrap(bwrap, secs, interpreter, script),
            None => {
                self.warn_once();
                self.run_plain(secs, interpreter, script)
            }
        }
    }

    fn run_plain(&self, secs: &str, interpreter: &str, script: &str) -> io::Result<Output> {
        Command::new(&self.timeout_bin)
            .args([secs, interpreter, "-c", script])
            .current_dir(&self.dir)
            .env_clear()
            .env("HOME", &self.dir)
            .env("PATH", &self.path_env)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .output()
    }

    fn run_bwrap(
        &self,
        bwrap: &Path,
        secs: &str,
        interpreter: &str,
        script: &str,
    ) -> io::Result<Output> {
        let dir_str = self.dir.to_string_lossy().to_string();
        let timeout_str = self.timeout_bin.to_string_lossy().to_string();
        Command::new(bwrap)
            .args([
                "--ro-bind",
                "/",
                "/",
                "--dev",
                "/dev",
                "--proc",
                "/proc",
                "--tmpfs",
                "/tmp",
                "--bind",
                dir_str.as_str(),
                dir_str.as_str(),
                "--chdir",
                dir_str.as_str(),
                "--unshare-net",
                "--die-with-parent",
                "--",
                timeout_str.as_str(),
                secs,
                interpreter,
                "-c",
                script,
            ])
            .env_clear()
            .env("HOME", &self.dir)
            .env("PATH", &self.path_env)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .output()
    }

    /// Warn once per sandbox (i.e. once per run) rather than once per entry.
    fn warn_once(&self) {
        if self
            .warn_count
            .compare_exchange(0, 1, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            eprintln!(
                "bashrs corpus: bwrap not found on PATH -- running corpus entries with a \
                 scoped temp cwd/$HOME/PATH but without namespace isolation"
            );
        }
    }

    #[cfg(test)]
    pub(crate) fn warn_count_for_test(&self) -> u32 {
        self.warn_count.load(Ordering::SeqCst)
    }
}

impl Drop for Sandbox {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

/// The process-wide sandbox, created once and shared across the runner's
/// worker threads for the lifetime of the process (one `corpus run`
/// invocation is one process).
pub(crate) fn shared() -> &'static Sandbox {
    static SANDBOX: OnceLock<Sandbox> = OnceLock::new();
    SANDBOX.get_or_init(Sandbox::new)
}

/// Resolve one absolute directory per required tool against the host's real
/// `PATH`, deduplicated and joined with ':'. This -- not the caller's full
/// `PATH` -- is what sandboxed entries execute with.
fn minimal_path() -> String {
    let caller_path = std::env::var("PATH").unwrap_or_default();
    let mut dirs: Vec<String> = Vec::new();
    for tool in REQUIRED_TOOLS {
        let Some(parent) =
            which_in(tool, &caller_path).and_then(|p| p.parent().map(Path::to_path_buf))
        else {
            continue;
        };
        let s = parent.to_string_lossy().to_string();
        if !dirs.contains(&s) {
            dirs.push(s);
        }
    }
    if dirs.is_empty() {
        FALLBACK_PATH.to_string()
    } else {
        dirs.join(":")
    }
}

/// Search `search_path` (":"-separated) for an executable named `name`.
fn which_in(name: &str, search_path: &str) -> Option<PathBuf> {
    std::env::split_paths(search_path)
        .map(|dir| dir.join(name))
        .find(|candidate| candidate.is_file())
}

/// Search the current process's real `PATH` for an executable named `name`.
fn which(name: &str) -> Option<PathBuf> {
    let path = std::env::var("PATH").unwrap_or_default();
    which_in(name, &path)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[test]
    fn test_PMAT258_sandbox_bwrap_absent_falls_back_to_scoped_run() {
        let sandbox = Sandbox::new_without_bwrap();
        assert_eq!(sandbox.warn_count_for_test(), 0);

        let result = sandbox.run("2", "sh", "true").unwrap();
        assert_ne!(result.status.code(), Some(124));
        assert_eq!(sandbox.warn_count_for_test(), 1, "warn must fire once");

        // A second entry executed against the same sandbox must not warn again.
        let _ = sandbox.run("2", "sh", "true").unwrap();
        assert_eq!(
            sandbox.warn_count_for_test(),
            1,
            "warn must not repeat per entry"
        );
    }

    #[test]
    fn test_PMAT258_sandbox_dash_absent_reports_gracefully() {
        // Build a PATH containing only `sh`'s directory (real machines almost
        // always merge /bin and /usr/bin, so isolating "no dash" by directory
        // isn't possible on the host PATH -- instead point PATH at a curated
        // empty directory so neither sh nor dash resolve, and confirm the
        // *shape* of the outcome (a completed process, not a hang/timeout)
        // is preserved -- exactly what "graceful" means for callers that
        // treat any non-124 exit as pass.
        let empty_dir = std::env::temp_dir().join(format!(
            "bashrs-corpus-sandbox-test-empty-path-{}",
            std::process::id()
        ));
        let _ = std::fs::create_dir_all(&empty_dir);
        let path_env = empty_dir.to_string_lossy().to_string();

        let sandbox = Sandbox::with_parts_for_test(path_env, None);
        let result = sandbox.run("2", "dash", "true");

        match result {
            Ok(output) => assert_ne!(
                output.status.code(),
                Some(124),
                "unresolvable interpreter must not surface as a timeout"
            ),
            Err(e) => assert_ne!(
                e.kind(),
                io::ErrorKind::TimedOut,
                "unresolvable interpreter must not surface as a timeout"
            ),
        }

        let _ = std::fs::remove_dir_all(&empty_dir);
    }

    #[test]
    fn test_PMAT258_sandbox_shared_reuses_same_dir() {
        let a = shared().dir().to_path_buf();
        let b = shared().dir().to_path_buf();
        assert_eq!(a, b, "shared() must return the same sandbox across calls");
    }
}
