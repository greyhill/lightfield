extern crate lightfield;
extern crate getopts;
extern crate proust;
extern crate num;

use self::getopts::Options;
use std::env;
use self::lightfield::*;
use self::proust::*;
use self::num::{Float, FromPrimitive, ToPrimitive};

// usage example
// recon_fbp --scene scene.toml --angles 21 --basis dirac

fn print_usage(name: &String, opt: Options) {
    let brief = format!("Usage: {} [options]", name);
    print!("{}", opt.usage(&brief));
}

fn volume_fbp<F: Float + FromPrimitive + ToPrimitive>(geom: &LightVolume<F>,
                        mut imagers: Vec<Box<Imager<F, LightVolume<F>>>>,
                        measurements: &[&[F]],
                        queue: &CommandQueue) -> Result<Vec<F>, Error> {
    let mut tr = geom.zeros();

    // TODO - filter measurements
    let mut filtered_measurements = Vec::new();
    for m in measurements.iter() {
        filtered_measurements.push(m.to_owned());
    }

    // backproject filtered measurements
    let mut backprojected_images = Vec::new();
    for (im, m) in imagers.iter_mut().zip(filtered_measurements.iter()) {
        backprojected_images.push(try!(im.back_host(m, queue)));
    }

    // add backprojected measurements
    for m in backprojected_images.iter() {
        for (tr_i, m_i) in tr.iter_mut().zip(m.iter()) {
            *tr_i = *tr_i + *m_i;
        }
    }

    // mask zero backprojections
    // (this is a crude sort of support estimation)
    for m in backprojected_images.iter() {
        for (tr_i, m_i) in tr.iter_mut().zip(m.iter()) {
            if *m_i == F::zero() {
                *tr_i = F::zero();
            }
        }
    }

    // project new image
    let mut projected_images = Vec::new();
    for (im, m) in imagers.iter_mut().zip(backprojected_images.iter()) {
        projected_images.push(try!(im.forw_host(m, queue)));
    }

    // compute scale (from data-fidelity line search)
    let mut num = F::zero();
    let mut denom = F::zero();
    for p in projected_images.iter() {
        for pi in p.iter() {
            denom = denom + *pi * *pi;
        }
    }
    for tr_i in tr.iter() {
        num = num + *tr_i * *tr_i;
    }

    // apply scale
    let scale = num / denom;
    for tr_i in tr.iter_mut() {
        *tr_i = *tr_i * scale;
    }

    Ok(tr)
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

            let measurement_slices: Vec<&[f32]> = measurements.iter().map(|m| &m[..]).collect();
            println!("Running \"FBP\"");

            let x_fbp = volume_fbp(&geom, imagers, &measurement_slices, queue).expect("Error computing FBP");

            geom.save(&x_fbp, &scene.object.data_path).expect("Error saving image");

            println!("Done!");
        },
    }
}

