use std::io::Read;

mod graphic;

pub use graphic::Graphic;

pub fn main() {
    let mut f = std::fs::File::open("monsterdata_test.mon").unwrap();
    let mut buf = Vec::new();
    f.read_to_end(&mut buf).expect("file reading failed");

    //    let monster = rvg_capnp::get_root_as_graphic(&buf[..]);
    //    println!("{}", monster.main_axis());
    //    println!("{}", monster.three_dimensions());
    //    println!("{}", monster.aspect_ratio());
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
