use super::*;

#[cfg(feature = "celeste_derive")]
pub use celeste_derive::*;

pub enum BinElValue {
    Attribute(BinElAttr),
    Element(BinEl)
}

pub trait BinElType: Sized {
    fn into_binel(self) -> BinElValue;
    fn from_binel(binel: BinElValue) -> Option<Self>;
}

macro_rules! impl_primitive {
    ($attr:ident, $type:ty, $val:ty) => (
        impl BinElType for $type {
            fn into_binel(self) -> BinElValue {
                BinElValue::Attribute(BinElAttr::$attr(self as $val))
            }
    
            fn from_binel(binel: BinElValue) -> Option<Self> {
                match binel {
                    BinElValue::Attribute(BinElAttr::$attr(e)) => Some(e as $type),
                    _ => None
                }
            }
        }
    )
}

impl_primitive!(Bool, bool, bool);
impl_primitive!(Int, u8, i32);
impl_primitive!(Int, u16, i32);
impl_primitive!(Int, i8, i32);
impl_primitive!(Int, i16, i32);
impl_primitive!(Int, i32, i32);
impl_primitive!(Float, f32, f32);
impl_primitive!(Text, String, String);

impl BinElType for BinEl {
    fn into_binel(self) -> BinElValue {
        BinElValue::Element(self)
    }

    fn from_binel(binel: BinElValue) -> Option<Self> {
        match binel {
            BinElValue::Element(e) => Some(e),
            _ => None
        }
    }
}

impl BinElType for BinElAttr {
    fn into_binel(self) -> BinElValue {
        BinElValue::Attribute(self)
    }

    fn from_binel(binel: BinElValue) -> Option<Self> {
        match binel {
            BinElValue::Attribute(e) => Some(e),
            _ => None
        }
    }
}
