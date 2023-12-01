// remember to change the module name!
use scaffold::Config;
use std::process;
fn main() {
    // remember to change the module name!
    if let Err(e) = scaffold::run(Config::make()) {
        eprintln!("{}", e);
        process::exit(1);
    } else {
        process::exit(0);
    }
}
