use celeste::*;
use std::io::Cursor;

#[test]
fn change_package() {
    let map_bytes = include_bytes!("empty.bin");
    let mut map_bin = binel::parser::take_file::<Error>(map_bytes).unwrap().1;
    eprintln!("{:#?}", map_bin);
    assert_eq!(map_bin.package, "test");
    map_bin.package = "newpkg".to_string();
    let mut changed_buf = Cursor::new(vec![0; 512]);
    binel::writer::put_file(&mut changed_buf, &map_bin).unwrap();
    let changed_bin = binel::parser::take_file::<Error>(&changed_buf.get_ref()[..])
        .unwrap()
        .1;
    assert_eq!(changed_bin.package, "newpkg");
}
