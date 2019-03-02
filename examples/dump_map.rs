use celeste::{*, binel::{*, serialize::*}};

fn main() {
    env_logger::init();

    let map_bytes = include_bytes!("empty.bin");
    let map_bin = parser::take_file(map_bytes).unwrap().1;
    println!("{:#?}", map_bin); // pretty print
    let map_data = maps::Map::from_binel(BinElValue::Element(map_bin.root));
    println!("{:#?}", map_data); // pretty print
}
