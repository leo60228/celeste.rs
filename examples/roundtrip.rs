use celeste::{
    binel::{serialize::*, *},
    *
};
use error_chain::ChainedError;

fn main() -> std::io::Result<()> {
    env_logger::init();

    let map_bytes = include_bytes!("empty.bin");
    let map_bin = parser::take_file(map_bytes).unwrap().1;
    println!("{:#?}", map_bin); // pretty print
    let map_data = match maps::Map::from_binel(BinElValue::Element(map_bin.root)) {
        Ok(map) => map,
        Err(err) => {
            println!("{}", err.display_chain().to_string());
            return Ok(());
        }
    };
    println!("{:#?}", map_data);
    let roundtrip_bin = match map_data.into_binel() {
        BinElValue::Element(elem) => elem,
        _ => panic!("Didn't get element!")
    };
    let roundtrip_file = BinFile {
        root: roundtrip_bin,
        package: map_bin.package
    };
    let mut file = std::fs::File::create("roundtrip.bin")?;
    binel::writer::put_file(&mut file, &roundtrip_file)?;
    Ok(())
}
