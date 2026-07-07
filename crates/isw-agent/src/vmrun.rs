//! Thin async wrapper around the VMware `vmrun` (VIX) CLI.
//!
//! `vmrun` is the only VMware Workstation interface that performs snapshots and
//! guest file/command injection, so it is the agent's control plane. All guest
//! operations require VMware Tools running in the guest and a valid guest login.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow};
use r2s_isw::protocol::PowerOp;
use tokio::process::Command;

/// A resolved `vmrun` binary, always invoked with host type `ws` (Workstation).
#[derive(Clone, Debug)]
pub struct Vmrun {
  path: PathBuf,
}

/// Output of a raw `vmrun` invocation.
pub struct VmrunOutput {
  pub code: i32,
  pub stdout: String,
  pub stderr: String,
}

impl VmrunOutput {
  pub fn ok(&self) -> bool {
    self.code == 0
  }

  pub fn combined(&self) -> String {
    format!("{}{}", self.stdout, self.stderr).trim().to_owned()
  }
}

impl Vmrun {
  /// Resolve the `vmrun` binary: explicit override, then well-known install
  /// locations (Windows + Linux), then bare `vmrun` on PATH.
  pub fn detect(override_path: Option<&str>) -> Result<Self> {
    if let Some(p) = override_path {
      let p = PathBuf::from(p);
      if p.exists() {
        return Ok(Self { path: p });
      }
      return Err(anyhow!("configured vmrun path does not exist: {}", p.display()));
    }
    const CANDIDATES: &[&str] = &[
      r"C:\Program Files (x86)\VMware\VMware Workstation\vmrun.exe",
      r"C:\Program Files\VMware\VMware Workstation\vmrun.exe",
      "/usr/bin/vmrun",
      "/usr/local/bin/vmrun",
    ];
    for c in CANDIDATES {
      let p = Path::new(c);
      if p.exists() {
        return Ok(Self { path: p.to_path_buf() });
      }
    }
    // fall back to PATH resolution at exec time.
    Ok(Self {
      path: PathBuf::from("vmrun"),
    })
  }

  pub fn path(&self) -> &Path {
    &self.path
  }

  /// Run `vmrun -T ws <args...>` and capture output.
  async fn exec(&self, args: &[String]) -> Result<VmrunOutput> {
    let output = Command::new(&self.path)
      .arg("-T")
      .arg("ws")
      .args(args)
      .output()
      .await
      .with_context(|| format!("failed to spawn vmrun at {}", self.path.display()))?;
    Ok(VmrunOutput {
      code: output.status.code().unwrap_or(-1),
      stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
      stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    })
  }

  /// Guest-authenticated invocation: `vmrun -T ws -gu <u> -gp <p> <args...>`.
  async fn exec_guest(&self, user: &str, pass: &str, args: &[String]) -> Result<VmrunOutput> {
    let mut full = vec![
      "-gu".to_owned(),
      user.to_owned(),
      "-gp".to_owned(),
      pass.to_owned(),
    ];
    full.extend_from_slice(args);
    self.exec(&full).await
  }

  pub async fn power(&self, vmx: &str, op: PowerOp) -> Result<VmrunOutput> {
    let args = match op {
      PowerOp::Start => vec!["start".to_owned(), vmx.to_owned(), "nogui".to_owned()],
      PowerOp::StopSoft => vec!["stop".to_owned(), vmx.to_owned(), "soft".to_owned()],
      PowerOp::StopHard => vec!["stop".to_owned(), vmx.to_owned(), "hard".to_owned()],
      PowerOp::Reset => vec!["reset".to_owned(), vmx.to_owned(), "hard".to_owned()],
      PowerOp::Suspend => vec!["suspend".to_owned(), vmx.to_owned(), "hard".to_owned()],
    };
    self.exec(&args).await
  }

  pub async fn snapshot(&self, vmx: &str, name: &str) -> Result<VmrunOutput> {
    self
      .exec(&["snapshot".to_owned(), vmx.to_owned(), name.to_owned()])
      .await
  }

  pub async fn revert(&self, vmx: &str, name: &str) -> Result<VmrunOutput> {
    self
      .exec(&[
        "revertToSnapshot".to_owned(),
        vmx.to_owned(),
        name.to_owned(),
      ])
      .await
  }

  pub async fn copy_to_guest(
    &self, vmx: &str, user: &str, pass: &str, host_path: &str, guest_path: &str,
  ) -> Result<VmrunOutput> {
    self
      .exec_guest(
        user,
        pass,
        &[
          "copyFileFromHostToGuest".to_owned(),
          vmx.to_owned(),
          host_path.to_owned(),
          guest_path.to_owned(),
        ],
      )
      .await
  }

  pub async fn copy_from_guest(
    &self, vmx: &str, user: &str, pass: &str, guest_path: &str, host_path: &str,
  ) -> Result<VmrunOutput> {
    self
      .exec_guest(
        user,
        pass,
        &[
          "copyFileFromGuestToHost".to_owned(),
          vmx.to_owned(),
          guest_path.to_owned(),
          host_path.to_owned(),
        ],
      )
      .await
  }

  pub async fn file_exists_in_guest(
    &self, vmx: &str, user: &str, pass: &str, guest_path: &str,
  ) -> Result<bool> {
    let out = self
      .exec_guest(
        user,
        pass,
        &[
          "fileExistsInGuest".to_owned(),
          vmx.to_owned(),
          guest_path.to_owned(),
        ],
      )
      .await?;
    // vmrun prints "The file exists." / "The file does not exist." and sets exit code.
    Ok(out.ok() && out.combined().to_lowercase().contains("exists")
      && !out.combined().to_lowercase().contains("does not"))
  }

  pub async fn run_script_in_guest(
    &self, vmx: &str, user: &str, pass: &str, interpreter: &str, script: &str,
  ) -> Result<VmrunOutput> {
    self
      .exec_guest(
        user,
        pass,
        &[
          "runScriptInGuest".to_owned(),
          vmx.to_owned(),
          "-interactive".to_owned(),
          interpreter.to_owned(),
          script.to_owned(),
        ],
      )
      .await
  }

  pub async fn run_program_in_guest(
    &self, vmx: &str, user: &str, pass: &str, program: &str, args: &[String],
  ) -> Result<VmrunOutput> {
    let mut full = vec![
      "runProgramInGuest".to_owned(),
      vmx.to_owned(),
      "-interactive".to_owned(),
      program.to_owned(),
    ];
    full.extend_from_slice(args);
    self.exec_guest(user, pass, &full).await
  }

  /// The set of currently-running `.vmx` paths, per `vmrun list`.
  pub async fn list_running(&self) -> Result<Vec<String>> {
    let out = self.exec(&["list".to_owned()]).await?;
    // first line is "Total running VMs: N", remaining lines are vmx paths.
    Ok(
      out
        .stdout
        .lines()
        .skip(1)
        .map(|l| l.trim().to_owned())
        .filter(|l| !l.is_empty())
        .collect(),
    )
  }

  pub async fn guest_ip(&self, vmx: &str) -> Result<Option<String>> {
    let out = self
      .exec(&[
        "getGuestIPAddress".to_owned(),
        vmx.to_owned(),
        "-wait".to_owned(),
      ])
      .await?;
    if out.ok() {
      let ip = out.stdout.trim().to_owned();
      if ip.is_empty() || ip.to_lowercase().contains("error") {
        Ok(None)
      } else {
        Ok(Some(ip))
      }
    } else {
      Ok(None)
    }
  }
}
