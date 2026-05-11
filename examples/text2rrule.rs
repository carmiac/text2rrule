//! Example tool to test text to rrule conversion.

use std::io::{self, BufRead};

use text2rrule::{text2rrule, text2rrule_with_locale};
use tracing_subscriber::fmt;

const HELP: &str = "\
text2rrule

Convert a plain language description of a repeating event into a RFC 5545 RRULE.
  
USEAGE:
  text2rrule [FLAGS] [OPTIONS]
  
OPTIONS:
  -i, --input       Input string (defaults to stdin)
  -l, --locale      Locale string, e.g. 'en-uk', detects current locale if not given.

FLAGS:
  -v, --verbose     Print verbose output for debugging 
  -h, --help        Print this help

EXAMPLES:
  text2rrule -i \"every two weeks on Friday\"
  FREQ=WEEKLY;INTERVAL=2;BYDAY=FR
";

#[derive(Debug)]
struct AppArgs {
    input: Option<String>,
    locale: Option<String>,
    verbose: bool,
}

fn parse_args() -> Result<AppArgs, pico_args::Error> {
    let mut pargs = pico_args::Arguments::from_env();
    if pargs.contains(["-h", "--help"]) {
        print!("{}", HELP);
        std::process::exit(0);
    }
    let verbose = pargs.contains(["-v", "--verbose"]);
    let args = AppArgs {
        input: pargs.opt_value_from_str(["-i", "--input"])?,
        locale: pargs.opt_value_from_str(["-l", "--locale"])?,
        verbose,
    };
    Ok(args)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = parse_args()?;
    if args.verbose {
        fmt().with_max_level(tracing::Level::DEBUG).init();
    }

    // Get the input, either from the option or a single line from stdin.
    let input = args
        .input
        .unwrap_or_else(|| io::stdin().lock().lines().next().unwrap().unwrap());
    let output = if args.locale.is_some() {
        text2rrule_with_locale(&input, [args.locale.unwrap()].into_iter())?
    } else {
        text2rrule(&input)?
    };
    println!("{}", output);
    Ok(())
}
