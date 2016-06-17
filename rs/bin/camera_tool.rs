extern crate lightfield;
extern crate getopts;
extern crate toml;

use self::getopts::Options;
use std::env;
use self::lightfield::*;
use self::fs::File;
use std::io::Write;

// usage target
// camera_tool --camera plenoptic.toml --focus_at 500

fn print_usage(name: &String, opt: Options) {
    let brief = format!("Usage: {} [options]", name);
    print!("{}", opts.usage(&brief));
}

fn main() {
    // get program name
    let args: Vec<String> = env::args().collect();
    let my_name = &args[0];

    // set up command line options parser
    let mut opts = Options::new();
    opts.reqopt("c", "camera", "TOML file describing a camera");
    opts.optopt("f", "focus_at", "Focus the camera at a given distance");
    opts.optflag("h", "help", "Print help and exit");

    // parse options
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            print_usage(my_name, opts);
            panic!(f.to_string());
        }
    };

    // if help requested, display and exit
    if matches.opt_present("h") {
        print_usage(my_name, opts);
        return;
    }

    // read camera
}
