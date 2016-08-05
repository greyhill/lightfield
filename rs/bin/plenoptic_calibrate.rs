extern crate lightfield;
extern crate getopts;
extern crate toml;

use self::getopts::Options;
use std::env;
use self::lightfield::*;
use self::toml::*;

// usage target:
// plenoptic_calibrate --camera camera.toml --points points.toml

fn setup_options() -> Options {
    let mut opts = Options::new();
    opts.reqopt("c", "camera", "TOML file describing camera", "TOMLFILE");
    opts.reqopt("p",
                "points",
                "TOML file containing point correspondences",
                "TOMLFILE");
    opts.reqopt("d",
                "distance",
                "Estimate of the distance to the points from the main lens",
                "FLOAT");
    opts.optflag("t", "depth", "Estimate depth to marked points");
    opts.optflag("i", "internal", "Estimate internal camera distances");
    opts.optflag("h", "help", "Print help and exit");
    opts
}

fn print_usage(name: &String, opts: Options) {
    let brief = format!("Usage: {} [options]", name);
    print!("{}", opts.usage(&brief));
}

fn main() {
    // get program name
    let args: Vec<String> = env::args().collect();
    let my_name = &args[0];

    // set up command line options parser
    let opts = setup_options();
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            print_usage(my_name, opts);
            panic!(f.to_string());
        }
    };

    // if requested, display help and exit
    if matches.opt_present("h") {
        print_usage(my_name, opts);
        return;
    }

    // read camera configuration
    let camera = {
        let mut config = CameraConfig::<f32>::from_map(&table_from_file(matches.opt_str("camera")
                    .unwrap())
                .expect("Error reading config"))
            .expect("Error parsing camera config");
        config.load_assets(matches.opt_str("camera").unwrap())
            .expect("Error loading camera assets");
        if let CameraConfig::PlenopticCamera(cam) = config {
            cam
        } else {
            panic!("Given camera is not a Plenoptic camera!");
        }
    };

    let estimate_depth = matches.opt_present("depth");
    let estimate_internal = matches.opt_present("internal");

    // read point correspondences
    //
    // the following looks a little hairy, so here's some explanation:
    // the coordinates file contains multiple lists of coordinates.  each
    // coordinate is a list: [ is, it ].  altogether, the file can be expressed
    // as a Vec<Vec<Vec<usize>>>, which we parse into a 
    // Vec<Vec<(usize, usize)>>.
    let points = {
        let toml = table_from_file(matches.opt_str("points").unwrap())
            .expect("Error loading coordinate file");
        if let Some(&Value::Array(ref coordinates)) = toml.get("coordinates") {
            let mut points = Vec::new();
            for v in coordinates.iter() {
                if let &Value::Array(ref vv) = v {
                    let mut point_pixels = Vec::new();
                    for vvv in vv.iter() {
                        if let &Value::Array(ref vvvv) = vvv {
                            let is_raw = &vvvv[0];
                            let it_raw = &vvvv[1];

                            match (is_raw, it_raw) {
                                (&Value::Integer(is), &Value::Integer(it)) => {
                                    point_pixels.push((is as usize, it as usize));
                                },
                                _ => {
                                    panic!("Malformed coordinates file: coordinates should be integers");
                                },
                            }
                        } else {
                            panic!("Malformed coordinates file: should be array of array of arrays ;)");
                        }
                    }
                    if point_pixels.len() > 0 {
                        points.push(point_pixels);
                    }
                } else {
                    panic!("Malformed coordinates file: should be array of arrays");
                }
            }
            points
        } else {
            panic!("Malformed coordinates file: no coordinates field or not an array");
        }
    };

    let distance = matches.opt_str("distance").unwrap().parse().expect("Error parsing distance");
    let distance_estimates = vec![ distance; points.len() ];

    if let Some(calibration) = PlenopticCameraCalibration::new(&camera, &points, &distance_estimates,
                                                               estimate_internal,
                                                               estimate_depth) {
        println!("{:?}", calibration);
    } else {
        panic!("Error performing calibration");
    }
}

