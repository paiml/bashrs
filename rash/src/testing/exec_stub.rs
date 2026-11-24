//! Write an executable stub script for a test, without racing `exec`.
//!
//! `std::fs::write` followed by `Command::new(path)` is a race in a
//! multi-threaded test binary. While this process holds the write
//! descriptor, any other test thread that forks a child gives that child a
//! copy of it. Until that child reaches its own `exec`, the kernel refuses
//! to execute the file: `execve` returns ETXTBSY ("Text file busy"). The
//! spawn fails, and a test that reads "spawn failed" as "tool not
//! installed" then reports the wrong outcome.
//!
//! So this process never opens the stub for writing. A child `sh` does the
//! write and the chmod; the test process only holds a pipe to it.

#[cfg(unix)]
use std::io::Write;
use std::path::Path;
#[cfg(unix)]
use std::process::{Command, Stdio};

/// Write `body` to `path` and make it executable (0755).
#[cfg(unix)]
pub(crate) fn write_exec_stub(path: &Path, body: &str) {
    let mut child = Command::new("/bin/sh")
        .args(["-c", "cat > \"$1\" && chmod 755 \"$1\"", "sh"])
        .arg(path)
        .stdin(Stdio::piped())
        .spawn()
        .expect("spawn sh to write the stub");
    child
        .stdin
        .take()
        .expect("stub writer stdin")
        .write_all(body.as_bytes())
        .expect("send the stub body");
    let status = child.wait().expect("wait for the stub writer");
    assert!(status.success(), "stub writer failed: {status}");
}

/// Not unix: no fork, so no inherited descriptor and no race.
#[cfg(not(unix))]
pub(crate) fn write_exec_stub(path: &Path, body: &str) {
    std::fs::write(path, body).expect("write the stub");
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;

    #[test]
    fn test_exec_stub_is_written_and_runs() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let stub = dir.path().join("tool");
        write_exec_stub(&stub, "#!/bin/sh\nprintf 'ran %s' \"$1\"\nexit 7\n");
        let out = Command::new(&stub).arg("x").output().expect("stub spawns");
        assert_eq!(out.status.code(), Some(7));
        assert_eq!(String::from_utf8_lossy(&out.stdout), "ran x");
    }

    /// The race itself: many threads write a stub and execute it at once.
    /// With `std::fs::write` in place of the child writer, some spawn fails
    /// with ETXTBSY (os error 26).
    #[test]
    fn test_exec_stub_never_text_file_busy_under_parallel_spawns() {
        let handles: Vec<_> = (0..16)
            .map(|_| {
                std::thread::spawn(|| {
                    for _ in 0..25 {
                        let dir = tempfile::TempDir::new().expect("tempdir");
                        let stub = dir.path().join("tool");
                        write_exec_stub(&stub, "#!/bin/sh\nexit 0\n");
                        let status = Command::new(&stub).status();
                        assert!(
                            matches!(&status, Ok(s) if s.success()),
                            "stub must spawn and exit 0, got {status:?}"
                        );
                    }
                })
            })
            .collect();
        for h in handles {
            h.join().expect("no thread saw a failed spawn");
        }
    }
}
