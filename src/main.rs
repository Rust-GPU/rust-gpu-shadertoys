#[cfg(feature = "headless")]
mod headless;

#[cfg(not(feature = "headless"))]
mod windowed;

use std::error::Error;

#[cfg(feature = "headless")]
fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    headless::run()
}

#[cfg(not(feature = "headless"))]
fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    windowed::run()
}
