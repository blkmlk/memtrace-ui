#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)] // it's an example

mod prelude;
mod ui;

use crate::ui::MemInfo;
use anyhow::{anyhow, Context};
use clap::Parser;
use common::interpret::Interpreter;
use prelude::*;
use std::fs::File;
use std::io::BufWriter;
use std::path::{Path, PathBuf};
use std::{env, fs, io};

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

    let lib_path = PathBuf::from(cargo_home)
        .join("lib")
        .join("libmemtrack.dylib");

    load_lib_if_needed(&lib_path).context("failed to load library")?;

    let pid = std::process::id();
    let trace_filepath = format!("/tmp/{}.trace", pid);

    let mut interpret = Interpreter::new(&trace_filepath).context("failed to create trace file")?;

    let cwd = std::env::current_dir().context("failed to get current directory")?;

    interpret
        .exec(opt.cmd, opt.args, cwd, lib_path.to_str().unwrap())
        .context("failed to execute process")?;

    let parsed_data = common::parser::Parser::new()
        .parse_file(&trace_filepath)
        .context("failed to parse trace file")?;

    let data = MemInfo {
        app_name,
        data: parsed_data,
    };

    ui::run_ui(data).map_err(|e| anyhow!("{:?}", e))?;

    Ok(())
}

fn load_lib_if_needed(path: impl AsRef<Path>) -> Result<()> {
    if path.as_ref().is_file() {
        return Ok(());
    }
    println!("Loading flamegraph from {}", path.as_ref().display());

    fs::create_dir_all(path.as_ref().parent().unwrap()).context("failed to create dirs")?;

    let mut response = reqwest::blocking::get(
        "https://github.com/blkmlk/memtrack-rs/releases/download/v0.1.1/libmemtrack.dylib",
    )
    .context("failed to download libmemtrack.dylib")?;

    if !response.status().is_success() {
        anyhow::bail!(
            "failed to download libmemtrack.dylib. status: {}",
            response.status()
        );
    }

    let mut out_file = BufWriter::new(File::create(path).context("failed to create output file")?);

    io::copy(&mut response, &mut out_file).context("failed to write output file")?;

    println!("Successfully loaded libmemtrack.dylib");

    Ok(())
}
