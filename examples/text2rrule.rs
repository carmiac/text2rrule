//! Example tool to test text to rrule conversion.

use std::io::{self, BufRead};

use text2rrule::{text2rrule, text2rrule_with_locale};
use tracing_subscriber::fmt;

const HELP: &str = "\
text2rrule

Convert a plain language description of a repeating event into a RFC 5545 RRULE.
  
USEAGE:
  text2rrule [FLAGS] [OPTIONS] [INPUT]
  
FLAGS:
  -v, --verbose     Print verbose output for debugging 
  -h, --help        Print this help

OPTIONS:
  -l, --locale      Locale string, e.g. 'en-uk', detects current locale if not given.

ARGS:
 <INPUT>           String to convert. If missing will read a line from stdin.

EXAMPLES:
  text2rrule \"every two weeks on Friday\"
  FREQ=WEEKLY;INTERVAL=2;BYDAY=FR
";

#[derive(Debug)]
struct AppArgs {
    locale: Option<String>,
    verbose: bool,
    input: Option<String>,
}

fn parse_args() -> Result<AppArgs, pico_args::Error> {
    let mut pargs = pico_args::Arguments::from_env();
    if pargs.contains(["-h", "--help"]) {
        print!("{}", HELP);
        std::process::exit(0);
    }
    let verbose = pargs.contains(["-v", "--verbose"]);
    let args = AppArgs {
        locale: pargs.opt_value_from_str(["-l", "--locale"])?,
        verbose,
        input: pargs.free_from_str().ok(),
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
    let output = if let Some(locale) = args.locale {
        text2rrule_with_locale(&input, [locale])?
    } else {
        text2rrule(&input)?
    };
    println!("{}", output);
    Ok(())
}
