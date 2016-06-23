extern crate lightfield as lf;
extern crate getopts as go;
extern crate proust;
extern crate avsfld;

use go::Options;
use std::env;
use lf::Serialize;
use lf::ClBuffer;
use lf::Geometry;
use self::proust::*;

// usage, nominally:
// render_phantom --phantom phantom.toml --geometry geometry.toml --out blah.fld

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
    opts.reqopt("p", "phantom", "TOML file describing phantom", "FILE");
    opts.reqopt("g", "geometry", "TOML file describing geometry", "FILE");
    opts.reqopt("o", "out", "Output path", "FILE");
    opts.optopt("d", "device", "OpenCL device to use (default: 0)", "INT");
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

    // read configuration
    let geom: lf::LightVolume<f32> =
        lf::LightVolume::from_map(&lf::table_from_file(matches.opt_str("geometry").unwrap())
                                       .expect("Error reading geometry configuration"))
            .expect("Error parsing geometry configuration");

    let ellipses: Vec<lf::Ellipsoid<f32>> =
        Vec::<lf::Ellipsoid<_>>::from_map(&lf::table_from_file(matches.opt_str("phantom")
                                                                      .unwrap())
                                               .expect("Error reading phantom configuration"))
            .expect("Error parsing phantom configuration");

    // create environment
    let env = lf::Environment::new_easy().expect("Error creating OpenCL environment");

    // use selected device
    let device_id = match matches.opt_str("device") {
        Some(s) => s.parse().expect("Error parsing device number"),
        None => 0usize,
    };

    let queue = &env.queues[device_id];
    println!("Using device id {} (of {}): {}",
             device_id,
             env.queues.len(),
             queue.device()
                  .expect("Error getting device info")
                  .name()
                  .expect("Error getting device name"));

    // create renderer, put things on the GPU
    let mut renderer = lf::PhantomRenderer::new(geom.clone(), queue.clone())
                           .expect("Error creating phantom renderer");
    let ell_buf = ellipses.as_cl_buffer(&queue).expect("Error loading ellipses onto GPU");
    let mut vol = geom.zeros_buf(&queue).expect("Error creating zero buffer");

    // render ellipses
    renderer.render_ellipsoids(ellipses.len(), &ell_buf, &mut vol, &[])
            .expect("Error rendering ellipses")
            .wait()
            .expect("Error waiting for render");

    // read rendered ellipses
    let mut rendered_vol = geom.zeros();
    queue.read_buffer(&vol, &mut rendered_vol).expect("Error reading rendered phantom");
    geom.save(&rendered_vol, matches.opt_str("out").unwrap())
        .expect("Error writing rendered phantom");

    println!("Rendered phantom written to {}",
             matches.opt_str("out").unwrap());
}
