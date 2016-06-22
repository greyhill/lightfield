extern crate lightfield;
extern crate getopts;
extern crate proust;
extern crate avsfld;

use self::getopts::Options;
use std::env;
use self::lightfield::*;
use self::proust::*;

// usage example:
// generate_data --scene scene.toml --angles 48 --basis dirac 

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
    opts.reqopt("s", "scene", "TOML file describing scene", "FILE");
    opts.reqopt("a", "angles", "Angular discretization", "INT");
    opts.reqopt("b", "basis", "Angular basis function", "pillbox | dirac");
    opts.optopt("d", "device", "OpenCL device to use (default: 0)", "INT");
    opts.optopt("v", "view", "Project only a single view", "INT");
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

    // use selected device
    let device_id = match matches.opt_str("device") {
        Some(s) => s.parse().expect("Error parsing device number"),
        None => 0usize
    };

    let queue = &env.queues[device_id];
    println!("Using device id {} (of {}): {}",
        device_id, env.queues.len(),
        queue.device().expect("Error getting device info").name()
                      .expect("Error getting device name"));

    // load scene description, object descriptions
    let scene = Scene::<f32>::read(matches.opt_str("s").unwrap()).expect("Error loading scene file");
    let object_config: ObjectConfig<f32> = scene.object.get_config().expect("Error reading object configuration");

    // branch based on the type of object given
    match object_config {
        ObjectConfig::LightVolume(geom) => {
            let object_buf = geom.load(&scene.object.data_path).expect("Error loading object data");
            let object = queue.create_buffer_from_slice(&object_buf).expect("Error loading object onto GPU");

            // loop through cameras
            for scene_cam in scene.cameras.iter() {
                println!("Simulating data for camera {}", scene_cam.name);
                let config = scene_cam.get_config().expect("Error reading camera configuration");
                let mut imager = config.volume_imager(geom.clone(),
                                                      scene_cam.position.clone(),
                                                      scene_cam.rotation.clone(),
                                                      na,
                                                      basis.clone(),
                                                      queue.clone()).expect("Error creating Imager for camera");
                let mut img = imager.detector().zeros_buf(&queue).expect("Error creating GPU detector buffer");

                let views = match matches.opt_str("view") {
                    Some(view_str) => {
                        let view = view_str.parse().expect("Error parsing view");
                        println!("Only projecting view {}", view);
                        vec![view]
                    },
                    None => {
                        let mut tr = Vec::new();
                        for ia in 0 .. imager.angular_plane().na() {
                            tr.push(ia);
                        }
                        tr
                    },
                };

                // Perform projection
                imager.forw_subset(&object, &mut img, &views, &[])
                    .expect("Error projecting").wait()
                    .expect("Error waiting for projection to complete");

                // Read projection to host
                let mut img_buf = imager.detector().zeros();
                queue.read_buffer(&img, &mut img_buf).expect("Error reading projection");

                // Save result
                imager.detector().save(&img_buf, &scene_cam.data_path).expect("Error saving result");
            }
        },
    }
}

