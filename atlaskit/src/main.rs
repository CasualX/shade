use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Arg, ArgAction, Command, value_parser};

macro_rules! error {
	($($arg:tt)*) => {
		eprintln!("error: {}", format_args!($($arg)*))
	};
}

macro_rules! fail {
	($($arg:tt)*) => {{
		error!($($arg)*);
		return Err(());
	}};
}

mod build;
mod info;

fn main() -> ExitCode {
	let matches = Command::new("atlaskit")
		.about("Build and manipulate Shade texture atlases")
		.subcommand_required(true)
		.arg_required_else_help(true)
		.subcommand(Command::new("build")
			.about("Build an atlas from an INI manifest")
			.arg(Arg::new("manifest").value_name("MANIFEST").required(true).value_parser(value_parser!(PathBuf)))
			.arg(Arg::new("output").long("output").short('o').value_name("OUTPUT").required(true).value_parser(value_parser!(PathBuf)).help("output prefix for the .png and .json files"))
			.arg(Arg::new("msdfgen").long("msdfgen").value_name("PROGRAM").value_parser(value_parser!(PathBuf)).help("msdfgen executable used by SVG sprites"))
			.arg(Arg::new("msdf_atlas_gen").long("msdf-atlas-gen").value_name("PROGRAM").value_parser(value_parser!(PathBuf)).help("msdf-atlas-gen executable used by fonts"))
			.arg(Arg::new("temp_dir").long("temp-dir").value_name("DIR").value_parser(value_parser!(PathBuf)).help("new directory for processed intermediate images"))
			.arg(Arg::new("keep_intermediate").long("keep-intermediate").action(ArgAction::SetTrue).help("keep processed intermediate images after a successful build"))
			.arg(Arg::new("pretty").long("pretty").action(ArgAction::SetTrue).help("format the output JSON with indentation"))
			.arg(Arg::new("preview").long("preview").action(ArgAction::SetTrue).help("Write an easy-to-view <output>.preview.png")),
		)
		.subcommand(Command::new("info")
			.about("Show statistics about an atlas")
			.arg(Arg::new("atlas").value_name("ATLAS").required(true).value_parser(value_parser!(PathBuf)).help("atlas JSON file to inspect")),
		)
		.get_matches();

	let result = match matches.subcommand() {
		Some(("build", matches)) => build::run(
			matches.get_one::<PathBuf>("manifest").unwrap(),
			build::BuildOptions {
				output: matches.get_one::<PathBuf>("output").unwrap().clone(),
				msdfgen: matches.get_one::<PathBuf>("msdfgen").cloned(),
				msdf_atlas_gen: matches.get_one::<PathBuf>("msdf_atlas_gen").cloned(),
				temp_dir: matches.get_one::<PathBuf>("temp_dir").cloned(),
				keep_intermediate: matches.get_flag("keep_intermediate"),
				pretty: matches.get_flag("pretty"),
				preview: matches.get_flag("preview"),
			},
		),
		Some(("info", matches)) => info::run(matches.get_one::<PathBuf>("atlas").unwrap()),
		_ => unreachable!("clap enforces subcommands"),
	};

	if result.is_ok() { ExitCode::SUCCESS } else { ExitCode::FAILURE }
}
