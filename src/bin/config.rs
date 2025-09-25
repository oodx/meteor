//! Configuration inspection utility
//!
//! Shows the current build-time configuration limits

fn main() {
    println!("{}", meteor::config_summary());
}
