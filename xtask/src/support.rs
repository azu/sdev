use sha2::{Digest, Sha256};
use std::{
    env,
    error::Error,
    ffi::OsString,
    fs,
    io::Read,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
    sync::atomic::{AtomicU64, Ordering},
    thread,
    time::{Duration, Instant},
};

pub type Result<T> = std::result::Result<T, Box<dyn Error>>;

pub fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_owned()
}

pub fn run(command: &mut Command) -> Result<()> {
    let status = command.status()?;
    if !status.success() {
        return Err(format!("{command:?} failed: {status}").into());
    }
    Ok(())
}

pub fn output(command: &mut Command) -> Result<String> {
    let output = command.output()?;
    if !output.status.success() {
        return Err(format!("{command:?}: {}", String::from_utf8_lossy(&output.stderr)).into());
    }
    Ok(String::from_utf8(output.stdout)?)
}

pub fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

pub fn sidecar(path: &Path, extension: &str) -> PathBuf {
    let mut name: OsString = path.as_os_str().to_owned();
    name.push(extension);
    PathBuf::from(name)
}

pub struct Toolchain {
    pub channel: String,
    compiler: String,
}

impl Toolchain {
    pub fn new() -> Result<Self> {
        let source = fs::read_to_string(root().join("rust-toolchain.toml"))?;
        let channel = source
            .lines()
            .find_map(|line| {
                let (key, value) = line.split_once('=')?;
                (key.trim() == "channel").then(|| value.trim().trim_matches('"').to_owned())
            })
            .ok_or("Missing toolchain channel")?;
        let compiler =
            output(Command::new("rustup").args(["which", "--toolchain", &channel, "rustc"]))?;
        Ok(Self {
            channel,
            compiler: compiler.trim().to_owned(),
        })
    }

    pub fn cargo(&self) -> Command {
        let mut command = Command::new("rustup");
        command
            .args(["run", &self.channel, "cargo"])
            .current_dir(root())
            .env("RUSTC", &self.compiler);
        command
    }
}

pub struct TemporaryDirectory(pub PathBuf);

impl TemporaryDirectory {
    pub fn new(prefix: &str) -> Result<Self> {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        loop {
            let path = env::temp_dir().join(format!(
                "sdev-{prefix}-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            match fs::create_dir(&path) {
                Ok(()) => return Ok(Self(path)),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(error.into()),
            }
        }
    }
}

impl Drop for TemporaryDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

pub fn bounded_output(command: &mut Command, timeout: Duration) -> Result<Output> {
    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let mut stdout = child.stdout.take().unwrap();
    let mut stderr = child.stderr.take().unwrap();
    let stdout = thread::spawn(move || {
        let mut bytes = Vec::new();
        stdout.read_to_end(&mut bytes).map(|_| bytes)
    });
    let stderr = thread::spawn(move || {
        let mut bytes = Vec::new();
        stderr.read_to_end(&mut bytes).map(|_| bytes)
    });
    let start = Instant::now();
    let status = loop {
        if let Some(status) = child.try_wait()? {
            break status;
        }
        if start.elapsed() >= timeout {
            child.kill()?;
            child.wait()?;
            stdout.join().map_err(|_| "stdout reader panicked")??;
            stderr.join().map_err(|_| "stderr reader panicked")??;
            return Err(format!("Process exceeded {timeout:?}").into());
        }
        thread::sleep(Duration::from_millis(5));
    };
    Ok(Output {
        status,
        stdout: stdout.join().map_err(|_| "stdout reader panicked")??,
        stderr: stderr.join().map_err(|_| "stderr reader panicked")??,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checksum_matches_known_sha256_and_sidecars_keep_dotted_names() {
        assert_eq!(
            digest(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_eq!(
            sidecar(Path::new("sdev-v0.1.0-windows-x64.exe"), ".json"),
            Path::new("sdev-v0.1.0-windows-x64.exe.json")
        );
    }

    #[test]
    #[cfg(unix)]
    fn terminates_a_release_process_that_does_not_exit() {
        let start = Instant::now();
        let error =
            bounded_output(Command::new("sleep").arg("1"), Duration::from_millis(20)).unwrap_err();
        assert!(error.to_string().contains("exceeded"));
        assert!(start.elapsed() < Duration::from_secs(1));
    }
}
