extern crate lightfield;
extern crate getopts;
extern crate proust;

use self::getopts::Options;
use std::env;
use self::lightfield::*;
use self::proust::*;

// usage example:
// recon_fista --scene scene.toml --angles 21 --basis dirac

fn print_usage(name: &String, opt: Options) {
    let brief = format!("Usage: {} [options]", name);
    print!("{}", opt.usage(&brief));
}

fn main() {
    // get binary name
    let args: Vec<String> = env::args().collect();
    let my_name = &args[0];

    // set up command line options parser
    let mut opts = Options::new();
    opts.reqopt("s", "scene", "TOML file describing scene", "FILE");
    opts.reqopt("a", "angles", "Angular discretization", "INT");
    opts.reqopt("b", "basis", "Angular basis function", "pillbox | dirac");
    opts.optopt("i", "interval", "Save an image every N iterations (default 1)", "INT");
    opts.optopt("n", "niter", "Maximum number of iterations (default none)", "INT");
    opts.optopt("u", "subsets", "Number of view subsets for acceleration (default 1)", "INT");
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

    // parse number of angles, basis function
    let na: usize = matches.opt_str("angles").unwrap().parse().expect("Error parsing number of angles");
    let basis = match &matches.opt_str("basis").unwrap()[..] {
        "dirac" => AngularBasis::Dirac,
        "pillbox" => AngularBasis::Pillbox,
        _ => panic!("Invalid angular basis"),
    };

    // create opencl environment
    let env = Environment::new_easy().expect("Error starting OpenCL environment");
    let queue = &env.queues[0];

    // load scene description, object descriptions
    let scene = Scene::<f32>::read(matches.opt_str("s").unwrap()).expect("Error loading scene file");
    let object_config: ObjectConfig<f32> = scene.object.get_config().expect("Error reading object configuration");

    // read iteration save interval
    let interval = match matches.opt_str("interval") {
        Some(s) => s.parse().expect("Error parsing interval"),
        None => 1usize,
    };

    // parse maximum number of iterations
    let niter: Option<usize> = match matches.opt_str("niter") {
        Some(s) => Some(s.parse().expect("Error parsing niter")),
        None => None,
    };

    // parse number of subsets
    let nsubset = match matches.opt_str("subsets") {
        Some(s) => s.parse().expect("Error parsing number of subsets"),
        None => 1usize
    };

    // branch based on the type of object given
    match object_config {
        ObjectConfig::LightVolume(geom) => {
            // loop through cameras
            let mut imagers = Vec::new();
            let mut measurements = Vec::new();
            for scene_cam in scene.cameras.iter() {
                // create imager object for camera
                println!("Loading camera {}", scene_cam.name);
                let config = scene_cam.get_config().expect("Error reading camera configuration");
                let imager = config.volume_imager(geom.clone(),
                                                  scene_cam.position.clone(),
                                                  scene_cam.rotation.clone(),
                                                  na,
                                                  basis.clone(),
                                                  queue.clone()).expect("Error creating Imager for camera");

                // load data
                let detector_ig = imager.detector().image_geometry();
                let meas = detector_ig.load(&scene_cam.data_path).expect("Error reading measurements");

                measurements.push(meas);
                imagers.push(imager);
            }

            // create fista solver
            let measurement_slices: Vec<&[f32]> = measurements.iter().map(|m| &m[..]).collect();
            println!("Initializing FISTA solver");
            let mut solver = FistaVolumeSolver::new(geom.clone(),
                                                    imagers,
                                                    &measurement_slices,
                                                    &scene.object.sparsifying,
                                                    nsubset,
                                                    scene.object.box_min,
                                                    scene.object.box_max,
                                                    queue.clone()).expect("Error creating FISTA solver");

            // loop iterations
            for iter in 0 .. {
                match niter {
                    Some(niter) => if niter == iter { break },
                    None => {},
                }

                // Run FISTA iteration
                println!("Starting iteration {}", iter);
                solver.run_subset(iter % nsubset, &[]).expect("Error running FISTA iteration").wait()
                    .expect("Error waiting for FISTA iteration to complete");

                // Get image
                if iter % interval == 0 {
                    let x_buf = solver.image_buffer();
                    let mut x = geom.zeros();
                    queue.read_buffer(&x_buf, &mut x).expect("Error reading image buffer").wait()
                        .expect("Error waiting waiting for image buffer transfer to complete");

                    geom.save(&x, &scene.object.data_path).expect("Error saving image");
                    println!("Saved image");
                }
            }

            println!("Done!");
        },
    }
}

