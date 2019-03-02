use crate::binel::*;
use crate::binel::serialize::*;

#[derive(Clone, PartialEq, Debug, Default, BinElType)]
#[celeste_name = "Style"]
pub struct Stylegrounds {
    pub foregrounds: Foregrounds,
    pub backgrounds: Backgrounds // TODO: implement stylegrounds
}

#[derive(Clone, PartialEq, Debug, Default, BinElType)]
#[celeste_name = "Foregrounds"]
pub struct Foregrounds {}

#[derive(Clone, PartialEq, Debug, Default, BinElType)]
#[celeste_name = "Backgrounds"]
pub struct Backgrounds {}

#[derive(Clone, PartialEq, Debug, Default, BinElType)]
#[celeste_name = "bgtiles"]
pub struct BGTiles {
    pub tileset: String
}

#[derive(Clone, PartialEq, Debug, Default, BinElType)]
#[celeste_name = "fgtiles"]
pub struct FGTiles {
    pub tileset: String
}

#[derive(Clone, PartialEq, Debug, Default, BinElType)]
pub struct Solids {
    #[celeste_name = "innerText"]
    pub contents: String
}

#[derive(Clone, PartialEq, Debug, Default, BinElType)]
#[celeste_name = "bg"]
pub struct BGSolids {
    #[celeste_name = "innerText"]
    pub contents: String
}

#[derive(Clone, PartialEq, Debug, Default, BinElType)]
#[celeste_name = "bgdecals"]
pub struct BGDecals {
    pub tileset: String
}

#[derive(Clone, PartialEq, Debug, Default, BinElType)]
#[celeste_name = "fgdecals"]
pub struct FGDecals {
    pub tileset: String
}

#[derive(Clone, PartialEq, Debug, Default, BinElType)]
pub struct Entities {
    #[celeste_child_vec]
    pub entities: Vec<BinEl>
}

#[derive(Clone, PartialEq, Debug, Default, BinElType)]
pub struct Triggers {
    #[celeste_child_vec]
    pub triggers: Vec<BinEl>
}

#[derive(Clone, PartialEq, Debug, Default, BinElType)]
#[celeste_name = "Filler"]
pub struct Filler {} // TODO: parse filler

#[derive(Clone, PartialEq, Debug, Default, BinElType)]
pub struct Level {
    pub name: String,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub music_layer_1: bool,
    pub music_layer_2: bool,
    pub music_layer_3: bool,
    pub music_layer_4: bool,
    pub music_progress: String,
    pub whisper: bool,
    pub underwater: bool,
    pub c: i32, // ???
    #[celeste_name = "alt_music"] // nice consistency there
    pub alt_music: String,
    pub space: bool,
    pub wind_pattern: String,
    pub disable_down_transition: bool,
    pub dark: bool,
    pub fgtiles: FGTiles,
    pub bgtiles: BGTiles,
    pub solids: Solids,
    pub bg: BGSolids,
}

#[derive(Clone, PartialEq, Debug, Default, BinElType)]
pub struct Levels {
    #[celeste_child_vec]
    pub levels: Vec<Level>
}

#[derive(Clone, PartialEq, Debug, Default, BinElType)]
#[celeste_name = "Map"]
pub struct Map {
    pub style: Stylegrounds,
    pub levels: Levels,
    pub filler: Filler
}

pub struct MapFile {
    pub package: String,
    pub map: Map
}
