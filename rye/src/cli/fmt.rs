use std::ffi::OsString;
use std::path::PathBuf;
use std::process::Command;

use anyhow::Error;
use clap::Parser;

use crate::bootstrap::ensure_self_venv;
use crate::consts::VENV_BIN;
use crate::pyproject::{locate_projects, PyProject};
use crate::utils::{CommandOutput, QuietExit};

/// Run the code formatter on the project.
///
/// This invokes ruff in format mode.
#[derive(Parser, Debug)]
pub struct Args {
    /// Format all packages
    #[arg(short, long)]
    all: bool,
    /// Format a specific package
    #[arg(short, long)]
    package: Vec<String>,
    /// Use this pyproject.toml file
    #[arg(long, value_name = "PYPROJECT_TOML")]
    pyproject: Option<PathBuf>,
    /// Run format in check mode
    #[arg(long)]
    check: bool,
    /// Enables verbose diagnostics.
    #[arg(short, long)]
    verbose: bool,
    /// Turns off all output.
    #[arg(short, long, conflicts_with = "verbose")]
    quiet: bool,
    /// Extra arguments to the formatter
    #[arg(trailing_var_arg = true)]
    extra_args: Vec<OsString>,
}

pub fn execute(cmd: Args) -> Result<(), Error> {
    let project = PyProject::load_or_discover(cmd.pyproject.as_deref())?;
    let output = CommandOutput::from_quiet_and_verbose(cmd.quiet, cmd.verbose);
    let venv = ensure_self_venv(output)?;
    let ruff = venv.join(VENV_BIN).join("ruff");

    let mut ruff_cmd = Command::new(ruff);
    ruff_cmd.arg("format");
    match output {
        CommandOutput::Normal => {}
        CommandOutput::Verbose => {
            ruff_cmd.arg("--verbose");
        }
        CommandOutput::Quiet => {
            ruff_cmd.arg("-q");
        }
    }

    if cmd.check {
        ruff_cmd.arg("--check");
    }
    ruff_cmd.args(cmd.extra_args);

    ruff_cmd.arg("--");
    let projects = locate_projects(project, cmd.all, &cmd.package[..])?;
    for project in projects {
        ruff_cmd.arg(project.root_path().as_os_str());
    }

    let status = ruff_cmd.status()?;
    if !status.success() {
        let code = status.code().unwrap_or(1);
        Err(QuietExit(code).into())
    } else {
        Ok(())
    }
}
