extern crate lightfield;
extern crate getopts;
extern crate toml;

use self::getopts::Options;
use std::env;
use self::lightfield::*;
use self::toml::*;
use std::fs::File;
use std::io::Write;

// usage target
// camera_tool --camera plenoptic.toml --focus_at 500

fn print_usage(name: &String, opts: Options) {
    let brief = format!("Usage: {} [options]", name);
    print!("{}", opts.usage(&brief));
}

fn main() {
    // get program name
    let args: Vec<String> = env::args().collect();
    let my_name = &args[0];

    // set up command line options parser
    let mut opts = Options::new();
    opts.reqopt("c", "camera", "TOML file describing a camera", "FILE");
    opts.optopt("f",
                "focus_at",
                "Focus the camera at a given distance",
                "DISTANCE");
    opts.optopt("o",
                "out",
                "(Optional) where to save the changed configuration",
                "FILE");
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

    // read camera configuration
    let mut camera = CameraConfig::<f32>::from_map(&table_from_file(matches.opt_str("camera")
                                                                           .unwrap())
                                                        .expect("Error reading config"))
                         .expect("Error parsing camera config");
    camera.load_assets(matches.opt_str("camera").unwrap()).expect("Error loading camera assets");

    // switched based on requested operation
    if matches.opt_present("focus_at") {
        let focus_distance: f32 = matches.opt_str("focus_at")
                                         .unwrap()
                                         .parse()
                                         .expect("Error parsing focus distance");
        println!("Focusing camera at {}", focus_distance);
        camera.focus_at_distance(focus_distance);
    }

    // if an output path is given, use it.  otherwise, modify the camera
    // configuration in place
    let out_path = match matches.opt_str("out") {
        Some(out) => out,
        None => matches.opt_str("camera").unwrap(),
    };

    // serialize modified camera config
    let modified_table = camera.into_map();
    let modified_toml = encode_str(&modified_table);

    // write ot file
    let mut out_file = File::create(out_path).expect("Error opening file for output");
    write!(&mut out_file, "{}", modified_toml).expect("Error writing modified configuration");
}
