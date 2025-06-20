mod prelude;
mod ui;

use crate::ui::MemInfo;
use anyhow::{anyhow, Context};
use clap::Parser;
use memtrack_utils::common::download_lib_if_needed;
use memtrack_utils::interpret::Interpreter;
use prelude::*;
use std::env;
use std::path::PathBuf;

const LIB_VERSION: &str = "v0.1.0";

#[derive(Parser)]
struct Opt {
    cmd: String,
    args: Vec<String>,
}

fn main() -> Result<()> {
    let opt = Opt::parse();

    let app_name = opt.cmd.split("/").last().unwrap().to_string();

    let Some(cargo_home) = env::var_os("CARGO_HOME") else {
        anyhow::bail!("missing $CARGO_HOME");
    };

    let lib_dir = PathBuf::from(cargo_home).join("lib");

    let lib_path =
        download_lib_if_needed(&lib_dir, LIB_VERSION).context("failed to load library")?;

    let pid = std::process::id();
    let trace_filepath = format!("/tmp/{}.trace", pid);

    let mut interpret = Interpreter::new(&trace_filepath).context("failed to create trace file")?;

    let cwd = std::env::current_dir().context("failed to get current directory")?;

    interpret
        .exec(opt.cmd, opt.args, cwd, &lib_path)
        .context("failed to execute process")?;

    let data = memtrack_utils::parser::Parser::new()
        .parse_file(&trace_filepath)
        .context("failed to parse trace file")?;

    let info = MemInfo { app_name, data };

    ui::run_ui(info).map_err(|e| anyhow!("{:?}", e))?;

    Ok(())
}
