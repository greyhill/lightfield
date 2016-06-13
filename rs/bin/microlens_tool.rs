extern crate lightfield;
extern crate getopts;
extern crate toml;

use self::getopts::Options;
use std::env;
use self::lightfield::*;
use self::toml::*;
use std::fs::File;
use std::io::Write;

// usage target:
// microlens_tool --detector detector.toml --pattern quad --out array.toml lens1.toml lens2.toml ...

fn print_usage(name: &String, opts: Options) {
    let brief = format!("Usage: {} [options] lens1.toml [lens2.toml ...]", name);
    print!("{}", opts.usage(&brief));
}

fn main() {
    // get program name
    let args: Vec<String> = env::args().collect();
    let my_name = &args[0];

    // set up command line options parser
    let mut opts = Options::new();
    opts.reqopt("g", "geometry", "TOML file describing plane geometry", "FILE");
    opts.reqopt("p", "pattern", "Pattern for lenses on the plane", "[quad]");
    opts.reqopt("o", "out", "Output TOML path", "FILE");
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

    // read plane geometry
    let plane_geom = ImageGeometry::<f32>::from_map(
        &table_from_file(matches.opt_str("geometry").unwrap()).expect("Error reading geometry")
    ).expect("Error parsing geometry");

    // read lenses
    let mut lenses: Vec<Lens<f32>> = Vec::new();
    for lens_path in matches.free.iter() {
        let lens = Lens::<f32>::from_map(
            &table_from_file(lens_path).expect("Error reading lens file")
        ).expect("Error parsing lens file");
        lenses.push(lens);
    }

    // tesselate the plane with lenses
    let lens_array: Vec<Lens<f32>> = match (&matches.opt_str("pattern").unwrap()[..], lenses.len()) {
        ("quad", 1) => Lens::tesselate_quad_1(&plane_geom, &lenses[0]),
        ("quad", 2) => Lens::tesselate_quad_2(&plane_geom, &lenses[0], &lenses[1]),
        _ => panic!("Unrecognized pattern and number of lenses"),
    };

    // convert lens array to TOML table and thence to String
    let lens_table = lens_array.into_map();
    let lens_string = encode_str(&lens_table);

    let mut out_file = File::create(matches.opt_str("out").unwrap()).expect("Error opening output file");
    write!(&mut out_file, "{}", lens_string).expect("Error writing output to file");

    println!("Saved a microlens array with {} lenses", lens_array.len());
}

