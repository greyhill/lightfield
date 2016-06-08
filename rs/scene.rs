extern crate num;
extern crate toml;
extern crate nalgebra as na;
use self::toml::*;
use std::path::*;
use isometry::*;
use self::na::{Rotation3, Vector3};
use std::fs::File;
use std::io::Read;
use self::num::{Float, ToPrimitive, FromPrimitive};
use object::*;
use camera::*;
use serialize::*;

fn path_from<P: AsRef<Path>, M: AsRef<Path>>(root_path: P, more: M) -> PathBuf {
    let mut tr = PathBuf::from(root_path.as_ref());
    tr.pop();
    tr.push(more);
    println!("derived path {:?}", tr);
    tr
}

pub fn table_from_file<P: AsRef<Path>>(path: P) -> Option<Table> {
    // open given path
    let mut file = match File::open(&path) {
        Ok(file) => file,
        Err(e) => {
            println!("Error opening TOML file: {}", e.to_string());
            return None;
        }
    };

    // read config as String
    let mut config_txt = String::new();
    if let Err(e) = file.read_to_string(&mut config_txt) {
        println!("Error reading config file: {}", e.to_string());
        return None;
    }

    // parse as toml
    let mut parser = Parser::new(&config_txt);
    let toml = match parser.parse() {
        Some(table) => table,
        None => {
            println!("Error parsing; malformed TOML");
            println!("Errors: {:?}", parser.errors);
            return None
        }
    };

    Some(toml)
}

/// Description of a camera from a configuration file
#[derive(Clone, Debug)]
pub struct SceneCamera<F: Float> {
    pub name: String,
    pub config: Table,
    pub data_path: Option<PathBuf>,
    pub position: Vector3<F>,
    pub rotation: Option<Rotation3<F>>,
}

impl<F: Float + FromPrimitive + ToPrimitive> SceneCamera<F> {
    pub fn get_config(self: &Self) -> Option<CameraConfig<F>> {
        CameraConfig::from_map(&self.config)
    }

    fn from_toml<P: AsRef<Path>>(root_path: P, table: &Table) -> Option<Self> {
        let name = if let Some(&Value::String(ref name)) = table.get("name") {
            name.to_owned()
        } else {
            println!("camera was given without a valid name");
            return None;
        };

        let config = match table.get("config") {
            Some(&Value::String(ref config_path_ext)) => {
                let path = path_from(&root_path, config_path_ext);
                if let Some(config) = table_from_file(path) {
                    config
                } else {
                    println!("Error parsing Camera config");
                    return None;
                }
            },
            Some(&Value::Table(ref tab)) => {
                tab.clone()
            },
            _ => {
                println!("Camera was listed without a valid configuration");
                return None;
            },
        };

        let data_path = match table.get("data") {
            Some(&Value::String(ref data_path_ext)) => {
                Some(path_from(&root_path, data_path_ext))
            },
            None => {
                None
            },
            _ => {
                println!("Camera data can be either unlisted or a path");
                return None;
            },
        };

        let position = match table.get("position") {
            Some(&Value::Table(ref tab)) => {
                if let Some(v) = Vector::<F>::from_map(tab) {
                    v
                } else {
                    println!("Malformed camera position");
                    return None;
                }
            },
            _ => {
                println!("Camera was not given a position");
                return None;
            },
        };

        Some(SceneCamera{
            name: name,
            config: config,
            data_path: data_path,
            position: position,
            rotation: None,
        })
    }
}

/// Description of object from a configuration file
#[derive(Clone, Debug)]
pub struct SceneObject {
    pub config: Table,
    pub data_path: Option<PathBuf>,
}

impl SceneObject {
    pub fn get_config<F: Float + FromPrimitive + ToPrimitive>(self: &Self) -> Option<ObjectConfig<F>> {
        ObjectConfig::from_map(&self.config)
    }

    fn from_toml<P: AsRef<Path>>(root_path: P, table: &Table) -> Option<Self> {
        let config = match table.get("config") {
            Some(&Value::String(ref path_ext)) => {
                let config_path = path_from(&root_path, path_ext);
                if let Some(config) = table_from_file(&config_path) {
                    config
                } else {
                    return None;
                }
            },
            Some(&Value::Table(ref tab)) => {
                tab.clone()
            },
            _ => {
                return None;
            },
        };

        let data_path = match table.get("data") {
            Some(&Value::String(ref path_ext)) => Some(path_from(&root_path, path_ext)),
            None => None,
            _ => {
                println!("object.data field was not a String?");
                return None;
            },
        };

        Some(SceneObject{
            config: config,
            data_path: data_path,
        })
    }
}

/// Description of a scene from a configuration file
#[derive(Clone, Debug)]
pub struct Scene<F: Float> {
    pub object: SceneObject,
    pub cameras: Vec<SceneCamera<F>>,
}

impl<F: Float + FromPrimitive> Scene<F> {
    pub fn read<P: AsRef<Path>>(path: P) -> Option<Self> {
        // open config file
        let config_toml = if let Some(toml) = table_from_file(&path) {
            toml
        } else {
            println!("Couldn't read config TOML");
            return None;
        };

        // get object table
        let object_table = match config_toml.get("object") {
            Some(&Value::Table(ref tab)) => tab,
            None => {
                println!("Config contained no [object] fields");
                return None;
            },
            _ => {
                println!("Config contained invalid [object] field");
                return None;
            },
        };

        // parse object description
        let object_desc = match SceneObject::from_toml(&path, &object_table) {
            Some(desc) => desc,
            None => {
                println!("Invalid object description in [object] table");
                return None;
            },
        };

        // get camera array
        let camera_array = match config_toml.get("camera") {
            Some(&Value::Array(ref arr)) => arr,
            None => {
                println!("Config contained no [[camera]] array");
                return None;
            },
            _ => {
                println!("Config contained invalid [[camera]] array");
                return None;
            },
        };

        // parse camera descriptions, dropping ones that don't parse
        // correctly (with a warning in the console)
        let mut camera_descs = Vec::new();
        for cam_val in camera_array.iter() {
            let cam_tab = match cam_val {
                &Value::Table(ref cam_tab) => cam_tab,
                _ => {
                    println!("Camera field was not table?");
                    continue;
                },
            };
            let cam_desc = match SceneCamera::from_toml(&path, cam_tab) {
                Some(cam_desc) => cam_desc,
                None => {
                    println!("Error parsing camera description; dropping");
                    continue;
                },
            };
            camera_descs.push(cam_desc);
        }

        // if we correctly parsed no cameras, that's an error
        if camera_descs.len() == 0 {
            println!("No valid camera descriptions parsed");
            return None;
        }

        // if we've gotten this far, we've succeeded :-)
        Some(Scene{
            object: object_desc,
            cameras: camera_descs,
        })
    }
}

#[test]
fn test_scene() {
    let scene = Scene::<f32>::read("cfg/test_scene.toml").unwrap();
    
    // PathBuf doesn't canonicalize automatically
    //assert_eq!(scene.object.data_path, Some(PathBuf::from("test_volume.fld")));

    assert_eq!(scene.cameras[0].name, "focal0");
    assert_eq!(scene.cameras[1].name, "focal1");

    assert_eq!(scene.cameras[0].position.x, 0.0);
    assert_eq!(scene.cameras[0].position.y, 0.0);
    assert_eq!(scene.cameras[0].position.z, -500.0);

    assert_eq!(scene.cameras[1].position.x, 50.0);
    assert_eq!(scene.cameras[1].position.y, 60.0);
    assert_eq!(scene.cameras[1].position.z, 12.0);

    if let Some(CameraConfig::SingleLensCamera(slc)) = scene.cameras[0].get_config() {
        assert_eq!(slc.detector.ns, 1024);
        assert_eq!(slc.detector.nt, 2048);
        assert_eq!(slc.detector.ds, 1.0);
        assert_eq!(slc.detector.dt, 2.0);
        assert_eq!(slc.detector.offset_s, 0.0);
        assert_eq!(slc.detector.offset_t, 1.0);
    } else {
        assert!(false);
    }
}

