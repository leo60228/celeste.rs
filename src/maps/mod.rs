use crate::binel::serialize::*;
use crate::binel::*;
use std::prelude::v1::*;

/// A `Level`'s "stylegrounds," or complexly animated backgrounds.
#[derive(Clone, PartialEq, Debug, Default, BinElType)]
#[celeste_name = "Style"]
pub struct Stylegrounds {
    pub foregrounds: Foregrounds,
    pub backgrounds: Backgrounds, // TODO: implement stylegrounds
}

/// Foreground stylegrounds. Currently not deserialized, instead just being a newtype over `BinEl`.
#[derive(Clone, PartialEq, Debug, Default, BinElType)]
#[celeste_name = "Foregrounds"]
pub struct Foregrounds(pub BinEl);

/// Background stylegrounds. Currently not deserialized, instead just being a newtype over `BinEl`.
#[derive(Clone, PartialEq, Debug, Default, BinElType)]
#[celeste_name = "Backgrounds"]
pub struct Backgrounds(pub BinEl);

/// The tilesets used in the `Level`'s background.
#[derive(Clone, PartialEq, Debug, Default, BinElType)]
#[celeste_name = "bgtiles"]
pub struct BGTiles {
    /// The tileset. Seems to always be "Scenery," but this may change.
    pub tileset: String,
}

/// The tilesets used in the `Level`'s foreground.
#[derive(Clone, PartialEq, Debug, Default, BinElType)]
#[celeste_name = "fgtiles"]
pub struct FGTiles {
    /// The tileset. Seems to always be "Scenery," but this may change.
    pub tileset: String,
}

/// The solid tiles in the `Level`'s foreground.
#[derive(Clone, PartialEq, Debug, Default, BinElType)]
pub struct Solids {
    /// The actual tiles, stored as a string.
    #[celeste_name = "innerText"]
    pub contents: String,
}

/// The tiles in the `Level`'s background.
#[derive(Clone, PartialEq, Debug, Default, BinElType)]
#[celeste_name = "bg"]
pub struct BGSolids {
    #[celeste_name = "innerText"]
    pub contents: String,
}

/// Decals, or image assets in a `Level`.
#[derive(Clone, PartialEq, Debug, Default, BinElType)]
pub struct Decal {
    /// The pixel (8 per tile) location in the `Level`.
    pub x: i32,
    /// The pixel (8 per tile) location in the `Level`.
    pub y: i32,
    /// Horizontal stretching of the `Decal`.
    pub scale_x: i32,
    /// Vertical stretching of the `Decal`.
    pub scale_y: i32,
    /// The texture of the `Decal`, within the Gameplay atlas.
    pub texture: String,
}

/// Background decals, or image assets in a `Level`.
#[derive(Clone, PartialEq, Debug, Default, BinElType)]
#[celeste_name = "bgdecals"]
pub struct BGDecals {
    /// The "tileset". Seems to always be "Scenery," but this may change. Unclear what this is used
    /// for.
    pub tileset: String,
    /// The decals.
    #[celeste_child_vec]
    pub decals: Vec<Decal>,
}

/// Foreground decals, or image assets in a `Level`.
#[derive(Clone, PartialEq, Debug, Default, BinElType)]
#[celeste_name = "fgdecals"]
pub struct FGDecals {
    /// The "tileset". Seems to always be "Scenery," but this may change. Unclear what this is used
    /// for.
    pub tileset: String,
    /// The decals.
    #[celeste_child_vec]
    pub decals: Vec<Decal>,
}

/// Entities, or objects in the `Level` with associated code.
#[derive(Clone, PartialEq, Debug, Default, BinElType)]
pub struct Entities {
    /// The actual entities. May be one of over 100 elements, so currently parsed as a raw BinEl.
    #[celeste_child_vec]
    pub entities: Vec<BinEl>,
}

/// Triggers, or regions in the `Level` with associated code.
#[derive(Clone, PartialEq, Debug, Default, BinElType)]
pub struct Triggers {
    /// The actual triggers. May be one of over 50 elements, so currently parsed as a raw BinEl.
    #[celeste_child_vec]
    pub triggers: Vec<BinEl>,
}

/// Object tiles. Poorly documented.
#[derive(Clone, PartialEq, Debug, Default, BinElType)]
pub struct ObjTiles {
    #[celeste_name = "innerText"]
    pub tiles: String,
}

/// Filler regions in the map. An alternate way of storing rooms filled with a single tile and no
/// other assets. Not currently parsed, as they are not necessary for most use cases. The `Map`'s
/// behavior shouldn't change if you remove these.
#[derive(Clone, PartialEq, Debug, Default, BinElType)]
#[celeste_name = "Filler"]
pub struct Filler(pub BinEl); // TODO: parse filler

/// Undocumented (apart from source) Everest extension, for storing the `Map`'s name and icon.
#[derive(Clone, PartialEq, Debug, Default, BinElType)]
pub struct Meta(pub BinEl);

/// A room in a `Map`. Only confusingly named fields are documented.
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
    /// Unclear what this does.
    pub c: i32, // ???
    /// Unclear what this does, though I believe that it is the default music used by Music
    /// triggers.
    #[celeste_name = "alt_music"] // nice consistency there
    pub alt_music: String,
    /// Alternate gravity. Behavior may be unstable in different game versions.
    pub space: bool,
    pub wind_pattern: String,
    pub disable_down_transition: bool,
    /// Affects lighting shaders.
    pub dark: bool,
    pub fgdecals: FGDecals,
    pub bgdecals: BGDecals,
    pub fgtiles: FGTiles,
    pub bgtiles: BGTiles,
    pub solids: Solids,
    pub bg: BGSolids,
    pub entities: Entities,
    pub triggers: Triggers,
    /// Optional, as some serializers (such as older versions of Maple) don't include these.
    pub objtiles: Option<ObjTiles>,
    /// All children that failed to parse.
    #[celeste_child_vec]
    pub invalid: Vec<BinEl>,
}

/// All `Level`s in a `Map`. Stored as a subelement for unknown reasons.
#[derive(Clone, PartialEq, Debug, Default, BinElType)]
pub struct Levels {
    #[celeste_child_vec]
    pub levels: Vec<Level>,
    /// All children that couldn't be parsed as `Level`s.
    #[celeste_child_vec]
    pub invalid_levels: Vec<BinEl>,
}

/// A chapter, also known as an Area in the game's code. `Map` was chosen to avoid confusion.
/// Parsed via the `BinElType` trait.
#[derive(Clone, PartialEq, Debug, Default, BinElType)]
#[celeste_name = "Map"]
pub struct Map {
    pub style: Stylegrounds,
    pub levels: Levels,
    pub filler: Filler,
    /// Optional, as it is an Everest extension, and thus many maps do not include it.
    pub meta: Option<Meta>,
}
