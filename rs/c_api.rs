use env::*;

use std::ptr;

#[no_mangle]
#[allow(non_snake_case)]
/// C API to create a new `Environment` object
pub unsafe fn LFEnvironment_new() -> *mut Environment {
    match Environment::new_easy() {
        Ok(env) => {
            Box::into_raw(Box::new(env))
        },
        Err(e) => {
            println!("Error creating environment: {:?}", e);
            ptr::null_mut()
        },
    }
}

#[no_mangle]
#[allow(non_snake_case)]
/// C API to destroy an `Environment` object
pub unsafe fn LFEnvironment_del(env: *mut Environment) {
    Box::from_raw(env);
}

