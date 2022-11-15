use pico_args::{Error, Arguments};

#[derive(Debug)]
pub struct Config {
	pub server: Option<String>,
}

pub fn parse_args() -> Option<Config> {
	let mut pargs = Arguments::from_env();

	if pargs.contains(["-h", "--help"]) {
		print_help();
		return None;
	}

	if pargs.contains(["-v", "--version"]) {
		print_version();
		return None;
	}

	let config = match inner_parse_args(&mut pargs) {
		Ok(config) => Some(config),
		Err(e) => {
			eprintln!("{}", e);
			return None;
		}
	};

	let remaining = pargs.finish();
	if !remaining.is_empty() {
		println!("Extraneous arguments provided: {:?}.", remaining);
		println!();
		print_try_help();
		return None;
	}

	config
}

fn inner_parse_args(pargs: &mut Arguments) -> Result<Config, Error> {
	Ok(Config {
		server: pargs.opt_value_from_str(["-s", "--server"])?,
	})
}

fn app_name() -> &'static str {
	env!("CARGO_PKG_NAME")
}

fn app_version() -> &'static str {
	env!("CARGO_PKG_VERSION")
}

fn print_usage() {
	println!("USAGE:");
	println!("  {} [OPTION] EXPRESSIONS", app_name());
}

const OPTIONS: &str = "\
FLAGS:
  -h, --help          print this help menu
  -v, --version       print version information

OPTIONS:
  -s, --server ADDR   connect to the given server address
";

fn print_opts() {
	print!("{}", OPTIONS);
}

fn print_help() {
	println!("{}", app_name());
	println!();
	print_usage();
	println!();
	print_opts();
}

fn print_try_help() {
	print_usage();
	println!();
	println!("Try '{} --help' for more information.", app_name());
}

fn print_version() {
	println!("{} {}", app_name(), app_version());
}
