use svd_parser as svd;

use std::fs::File;
use std::io::Read;
fn main() {
    let xml = &mut String::new();
    File::open("nrf52840.svd").unwrap().read_to_string(xml);
    println!("{:?}", svd::parse(xml).unwrap().vendor);
}
