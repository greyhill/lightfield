extern crate proust;
use self::proust::*;

/// OpenCL context and objects
pub struct Environment {
    pub ctx: Context,
    pub devices: Vec<Device>,
    pub queues: Vec<CommandQueue>,
}

impl Environment {
    /// Create an Environment with the first platform available
    /// 
    /// Only uses GPU devices by default.
    pub fn new_easy() -> Result<Environment, Error> {
        let platforms = try!(Platform::platforms());
        let devices = try!(platforms[0].devices());
        let mut gpu_devices: Vec<Device> = devices.iter().map(|d| d.clone())
                                              .filter(|d| {
                                                  if let DeviceType::GPU = d.device_type()
                                                                            .unwrap() {
                                                      true
                                                  } else {
                                                      false
                                                  }
                                              })
                                              .collect();
        if gpu_devices.len() == 0 {
            gpu_devices = devices;
        }

        let context = try!(Context::new(&gpu_devices));
        let mut queues = Vec::new();
        for d in gpu_devices.iter() {
            let q = try!(CommandQueue::new(context.clone(), d.clone()));
            queues.push(q);
        }

        Ok(Environment {
            ctx: context,
            devices: gpu_devices,
            queues: queues,
        })
    }
}

#[test]
fn test_new_easy() {
    let env = Environment::new_easy().unwrap();
    assert!(env.devices.len() > 0);
}
