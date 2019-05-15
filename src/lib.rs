extern crate flatbuffers;

#[allow(dead_code, unused_imports)]
#[path = "../rvg_generated.rs"]
mod rvg_generated;
pub use rvg_generated::rvg;

use std::io::Read;

pub fn main() {
    let mut f = std::fs::File::open("monsterdata_test.mon").unwrap();
    let mut buf = Vec::new();
    f.read_to_end(&mut buf).expect("file reading failed");

    let monster = rvg::get_root_as_graphic(&buf[..]);
    println!("{}", monster.main_axis());
    println!("{}", monster.three_dimensions());
    println!("{}", monster.aspect_ratio());
}
