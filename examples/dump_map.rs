use celeste::{
    binel::{serialize::*, *},
    *
};
use error_chain::ChainedError;

fn main() {
    env_logger::init();

    let map_bytes = include_bytes!("empty.bin");
    let map_bin = parser::take_file(map_bytes).unwrap().1;
    println!("{:#?}", map_bin); // pretty print
    let map_data = match maps::Map::from_binel(BinElValue::Element(map_bin.root)) {
        Ok(map) => map,
        Err(err) => {
            println!("{}", err.display_chain().to_string());
            return;
        }
    };
    println!("{:#?}", map_data);
}
