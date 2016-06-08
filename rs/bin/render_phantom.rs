extern crate lightfield as lf;
extern crate getopts as go;

use go::Options;
use std::env;

// usage, nominally:
// render_volume --camera camera.toml --phantom phantom.toml --discretization disc.toml --distance 50 --out blah.png
//      --plane pinhole | pillbox --angles 20

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
    opts.reqopt("c", "camera", "TOML file describing camera", "FILE");
    opts.reqopt("p", "phantom", "TOML file describing phantom ", "FILE");
    opts.reqopt("z", "discretization", "TOML file describing discretization", "FILE");
    opts.reqopt("d", "distance", "Distance (mm) from the camera to the center of the scene", "DISTANCE");
    opts.reqopt("o", "out", "Where to save rendered image", "FILE");
    opts.reqopt("b", "basis", "Angular basis function", "pinhole | pillbox");
    opts.reqopt("a", "angles", "Number of angles", "INT");
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
}

