mod support;
mod main_instance;
mod utils;

use crate::utils::os_identity::identification;
use crate::main_instance::initialice_main_ui;

fn main() {
  let os = identification(); 
  println!("{:?}", os);
  initialice_main_ui();
}

