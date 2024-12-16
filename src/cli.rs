const HELP: &str = "\
App

USAGE:
  http-server [OPTIONS] --directory STRING [INPUT]

FLAGS:
  -h, --help            Prints help information

OPTIONS:
  --directory STRING   the directory where the files are stored, as an absolute path

ARGS:
  <INPUT>
";

#[derive(Debug)]
pub(crate) struct AppArgs {
    pub(crate) directory: Option<String>,
}

pub(crate) fn parse_args() -> Result<AppArgs, pico_args::Error> {
    let mut pargs = pico_args::Arguments::from_env();

    let args = AppArgs {
        directory: pargs.opt_value_from_str("--directory")?,
    };

    // It's up to the caller what to do with the remaining arguments.
    let remaining = pargs.finish();
    if !remaining.is_empty() {
        eprintln!("Warning: unused arguments left: {:?}.", remaining);
    }

    return Ok(args);
}
