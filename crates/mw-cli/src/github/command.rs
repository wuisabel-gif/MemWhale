//! Bounded subprocess capture. No reader threads: inherited pipes cannot make
//! cleanup hang in a join, even when a descendant escapes the process group.

use super::{external_text, MAX_GH_RESPONSE_BYTES};
use std::io::{self, Read};
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

pub(super) fn run(
    mut command: Command,
    stdout_limit: usize,
    runtime: Duration,
) -> Result<String, String> {
    // Release support is Linux, macOS, and WSL. Reject before spawning on
    // native Windows: synchronous std pipes cannot promise this deadline.
    if !cfg!(unix) {
        return Err(
            "GitHub context requires Linux, macOS, or WSL; native Windows is not supported"
                .to_string(),
        );
    }
    command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }
    let started = Instant::now();
    let mut child = command
        .spawn()
        .map_err(|error| format!("GitHub CLI (`gh`) is unavailable: {error}"))?;
    // Every error after spawn goes through the same kill/reap path, including
    // pipe setup, read, wait, and UTF-8 errors.
    let result = (|| {
        let mut stdout = Capture::new(
            child
                .stdout
                .take()
                .ok_or("GitHub CLI stdout is unavailable")?,
        )?;
        let mut stderr = Capture::new(
            child
                .stderr
                .take()
                .ok_or("GitHub CLI stderr is unavailable")?,
        )?;
        let mut status = None;
        loop {
            if started.elapsed() >= runtime {
                return Err(format!(
                    "GitHub CLI exceeded the {} ms runtime limit",
                    runtime.as_millis()
                ));
            }
            // One bounded read per pipe per turn keeps both streams and the
            // deadline responsive, including when stdout is continuously busy.
            let progress = stdout.step(stdout_limit, "stdout")?
                | stderr.step(MAX_GH_RESPONSE_BYTES, "stderr")?;
            if status.is_none() {
                status = child
                    .try_wait()
                    .map_err(|error| format!("failed waiting for GitHub CLI: {error}"))?;
            }
            if let Some(status) = status.filter(|_| stdout.closed && stderr.closed) {
                if !status.success() {
                    let detail = external_text(&String::from_utf8_lossy(&stderr.bytes), 1000);
                    return Err(if detail.is_empty() {
                        format!("GitHub CLI exited with {status}")
                    } else {
                        format!("GitHub CLI exited with {status}: {detail}")
                    });
                }
                return String::from_utf8(stdout.bytes)
                    .map_err(|_| "GitHub CLI returned non-UTF-8 output".to_string());
            }
            // EOF is not process exit: keep polling under the same deadline.
            if !progress {
                thread::sleep(
                    Duration::from_millis(10).min(runtime.saturating_sub(started.elapsed())),
                );
            }
        }
    })();
    if result.is_err() {
        stop_child(&mut child);
    }
    result
}

fn stop_child(child: &mut Child) {
    #[cfg(unix)]
    // SAFETY: the child was started in its own process group; a negative PID
    // addresses only that group, never MemoryWhale's group.
    unsafe {
        libc::kill(-(child.id() as libc::pid_t), libc::SIGKILL);
    }
    let _ = child.kill();
    // This wait follows forced termination, never a still-running normal child.
    let _ = child.wait();
}

struct Capture<R> {
    pipe: R,
    bytes: Vec<u8>,
    closed: bool,
}

impl<R: Pipe> Capture<R> {
    fn new(pipe: R) -> Result<Self, String> {
        pipe.prepare()
            .map_err(|error| format!("failed preparing GitHub CLI pipe: {error}"))?;
        Ok(Self {
            pipe,
            bytes: Vec::new(),
            closed: false,
        })
    }

    fn step(&mut self, limit: usize, stream: &str) -> Result<bool, String> {
        if self.closed {
            return Ok(false);
        }
        let mut buffer = [0_u8; 8192];
        match self.pipe.read_ready(&mut buffer) {
            Ok(0) => self.closed = true,
            Ok(count) => {
                if self.bytes.len().saturating_add(count) > limit {
                    return Err(format!(
                        "GitHub response exceeded the {limit} byte {stream} safety limit"
                    ));
                }
                self.bytes.extend_from_slice(&buffer[..count]);
                return Ok(true);
            }
            Err(error)
                if matches!(
                    error.kind(),
                    io::ErrorKind::WouldBlock | io::ErrorKind::Interrupted
                ) => {}
            Err(error) => return Err(format!("failed reading GitHub CLI {stream}: {error}")),
        }
        Ok(false)
    }
}

trait Pipe: Read {
    fn prepare(&self) -> io::Result<()>;
    fn read_ready(&mut self, buffer: &mut [u8]) -> io::Result<usize>;
}

#[cfg(unix)]
impl<R: Read + std::os::fd::AsRawFd> Pipe for R {
    fn prepare(&self) -> io::Result<()> {
        // SAFETY: the owned pipe remains open throughout both fcntl calls.
        unsafe {
            let flags = libc::fcntl(self.as_raw_fd(), libc::F_GETFL);
            if flags < 0
                || libc::fcntl(self.as_raw_fd(), libc::F_SETFL, flags | libc::O_NONBLOCK) < 0
            {
                return Err(io::Error::last_os_error());
            }
        }
        Ok(())
    }

    fn read_ready(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        self.read(buffer)
    }
}

#[cfg(not(unix))]
impl<R: Read> Pipe for R {
    fn prepare(&self) -> io::Result<()> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "bounded GitHub CLI capture is unsupported on this platform",
        ))
    }

    fn read_ready(&mut self, _buffer: &mut [u8]) -> io::Result<usize> {
        unreachable!("prepare rejects unsupported pipes")
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;

    struct BrokenPipe;

    impl Read for BrokenPipe {
        fn read(&mut self, _buffer: &mut [u8]) -> io::Result<usize> {
            Err(io::Error::other("injected read error"))
        }
    }

    impl Pipe for BrokenPipe {
        fn prepare(&self) -> io::Result<()> {
            Ok(())
        }

        fn read_ready(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
            self.read(buffer)
        }
    }

    #[test]
    fn read_errors_are_not_mistaken_for_successful_eof() {
        let mut capture = Capture::new(BrokenPipe).unwrap();
        assert!(capture
            .step(100, "stdout")
            .unwrap_err()
            .contains("injected read error"));
        assert!(!capture.closed);
    }
}
