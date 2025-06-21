//!
//! # MemTrace UI
//!
//! A GUI Rust-based tool for visualizing heap memory consumption inspired by [heaptrack](https://github.com/KDE/heaptrack). MemTrace supports heaptrack trace files, so you can read the samples built on Linux
//!
//! The tool is using the [egui](https://github.com/emilk/egui) crate for building UI
//!
//! > ℹ️ **Info:** So far, the tool works only on MacOS.
//!
//! > ⚠️ **Warning:** At the moment, this tool requires downloading a dynamic library to function. The library is open source and can be found [here](https://github.com/blkmlk/memtrace-lib).
//!
//! ## Supported features:
//!
//! ## 1. Overview:
//! ![overview](overview.png)
//!
//! ## 2. TopDown tree with source code:
//! ![topdown](topdown.png)
//!
//! ## 3. Flamegraph:
//! ![flamegraph](flamegraph.png)
//!

mod prelude;
mod ui;

use crate::ui::MemInfo;
use anyhow::{anyhow, Context};
use clap::Parser;
use memtrace_utils::common::download_lib_if_needed;
use memtrace_utils::interpret::Interpreter;
use prelude::*;
use std::env;
use std::path::PathBuf;

const LIB_VERSION: &str = "v0.2.0";

#[derive(Parser)]
struct Opt {
    cmd: String,
    args: Vec<String>,
}

fn main() -> Result<()> {
    let opt = Opt::parse();

    let app_name = opt.cmd.split("/").last().unwrap().to_string();

    let Some(home) = env::var_os("HOME") else {
        anyhow::bail!("missing $HOME");
    };

    let lib_dir = PathBuf::from(home).join(".cargo").join("lib");

    let lib_path =
        download_lib_if_needed(&lib_dir, LIB_VERSION).context("failed to load library")?;

    let pid = std::process::id();
    let trace_filepath = format!("/tmp/{}.trace", pid);

    let mut interpret = Interpreter::new(&trace_filepath).context("failed to create trace file")?;

    let cwd = std::env::current_dir().context("failed to get current directory")?;

    interpret
        .exec(opt.cmd, opt.args, cwd, &lib_path)
        .context("failed to execute process")?;

    let data = memtrace_utils::parser::Parser::new()
        .parse_file(&trace_filepath)
        .context("failed to parse trace file")?;

    let info = MemInfo { app_name, data };

    ui::run_ui(info).map_err(|e| anyhow!("{:?}", e))?;

    Ok(())
}
