#![allow(missing_docs)] // TODO: remove

use crate::{Error, Result};
use derive_into_owned::IntoOwned;
use derive_more::{From, Into};
use futures::prelude::*;
use nom::{
    bytes::streaming::{tag, take_until},
    combinator::{cond, flat_map, iterator, map, map_res},
    error::ParseError,
    multi::length_data,
    number::streaming::{le_u32, le_u64, le_u8},
    sequence::{terminated, tuple},
    IResult,
};
use smallvec::SmallVec;
use std::borrow::Cow;
use std::convert::TryFrom;
use std::fmt;
use std::str;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ChunkType<'a> {
    MChat,
    MPlayer,
    MRequest,
    MServerInfo,
    UUpdate,
    UActionCollision,
    UAudioPlay,
    Eof,
    HHead,
    Unknown(&'a str),
}

impl<'a> From<&'a str> for ChunkType<'a> {
    fn from(s: &'a str) -> Self {
        use ChunkType::*;
        match s {
            "nMC" => MChat,
            "nM" => MPlayer,
            "nMR" => MRequest,
            "nM?" => MServerInfo,
            "nU" => UUpdate,
            "nUaC" => UActionCollision,
            "nUAP" => UAudioPlay,
            "\r\n" => Eof,
            "nH" => HHead,
            s => Unknown(s),
        }
    }
}

impl<'a> From<ChunkType<'a>> for &'a str {
    fn from(c: ChunkType<'a>) -> Self {
        use ChunkType::*;
        match c {
            MChat => "nMC",
            MPlayer => "nM",
            MRequest => "nMR",
            MServerInfo => "nM?",
            UUpdate => "nU",
            UActionCollision => "nUaC",
            UAudioPlay => "nUAP",
            Eof => "\r\n",
            HHead => "nH",
            Unknown(s) => s,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, From, Default)]
pub struct MChat<'a> {
    pub id: u32,
    pub tag: &'a str,
    pub text: &'a str,
    pub red: u8,
    pub blue: u8,
    pub green: u8,
    pub date: u64,
}

impl<'a> MChat<'a> {
    pub fn parse(data: &'a [u8]) -> Result<(&'a [u8], Self)> {
        Ok(map(
            tuple((le_u32, null_str, null_str, le_u8, le_u8, le_u8, le_u64)),
            From::from,
        )(data)?)
    }

    pub async fn write(
        &'a self,
        stream: &mut (dyn AsyncWrite + Send + Sync + Unpin),
    ) -> Result<'a, ()> {
        let id_bytes = self.id.to_le_bytes();
        stream.write_all(&id_bytes).await?;
        stream.write_all(self.tag.as_ref()).await?;
        stream.write_all(&[0]).await?;
        stream.write_all(self.text.as_ref()).await?;
        stream.write_all(&[0]).await?;
        let rgb = [self.red, self.green, self.blue];
        stream.write_all(&rgb).await?;
        let date_bytes = self.date.to_le_bytes();
        stream.write_all(&date_bytes).await?;

        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, From, Into)]
pub struct UUpdate<'a>(&'a [u8]);

impl UUpdate<'_> {
    pub fn id(&self) -> u32 {
        u32::from_le_bytes(<_>::try_from(&self.0[..4]).unwrap())
    }

    pub fn remainder(&self) -> &[u8] {
        &self.0[4..]
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, From, Into)]
pub struct UAudioPlay<'a>(&'a [u8]);

impl UAudioPlay<'_> {
    pub fn bytes(&self) -> &[u8] {
        self.0
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, From, Into)]
pub struct UActionCollision<'a>(&'a [u8]);

impl UActionCollision<'_> {
    pub fn bytes(&self) -> &[u8] {
        self.0
    }
}

#[derive(Debug, PartialEq, Eq, Clone, From, Default, IntoOwned)]
pub struct MPlayer<'a> {
    pub echo: bool,
    pub name: Cow<'a, str>,
    pub area: Cow<'a, str>,
    pub mode: u8,
    pub level: Cow<'a, str>,
    pub completed: bool,
    pub exit: Option<u8>,
    pub idle: bool,
}

impl<'a> MPlayer<'a> {
    pub fn parse(data: &'a [u8]) -> Result<(&'a [u8], Self)> {
        Ok(map(
            tuple((
                boolean,
                map(null_str, Cow::Borrowed),
                map(null_str, Cow::Borrowed),
                le_u8,
                map(null_str, Cow::Borrowed),
                boolean,
                flat_map(boolean, |b| cond(b, le_u8)),
                boolean,
            )),
            From::from,
        )(data)?)
    }

    pub async fn write(
        &'a self,
        stream: &mut (dyn AsyncWrite + Send + Sync + Unpin),
    ) -> Result<'a, ()> {
        stream
            .write_all(if self.echo { &[1] } else { &[0] })
            .await?;
        stream.write_all(self.name.as_ref().as_ref()).await?;
        stream.write_all(&[0]).await?;
        stream.write_all(self.area.as_ref().as_ref()).await?;
        stream.write_all(&[0]).await?;
        let mode_byte = [self.mode];
        stream.write_all(&mode_byte).await?;
        stream.write_all(self.level.as_ref().as_ref()).await?;
        stream.write_all(&[0]).await?;
        stream
            .write_all(if self.completed { &[1] } else { &[0] })
            .await?;
        if let Some(exit) = self.exit {
            stream.write_all(&[1]).await?;
            let exit_byte = [exit];
            stream.write_all(&exit_byte).await?;
        } else {
            stream.write_all(&[0]).await?;
        }
        stream
            .write_all(if self.idle { &[1] } else { &[0] })
            .await?;
        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, From)]
pub struct HHead {
    pub id: u32,
}

impl HHead {
    pub fn parse<'a>(data: &'a [u8]) -> Result<'a, (&'a [u8], Self)> {
        Ok(map(le_u32, From::from)(data)?)
    }

    pub async fn write(
        &self,
        stream: &mut (dyn AsyncWrite + Send + Sync + Unpin),
    ) -> Result<'static, ()> {
        let id_bytes = self.id.to_le_bytes();
        stream.write_all(&id_bytes).await?;

        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, From)]
pub struct MRequest<'a> {
    pub id: ChunkType<'a>,
}

impl<'a> MRequest<'a> {
    pub fn parse(data: &'a [u8]) -> Result<'a, (&'a [u8], Self)> {
        Ok(map(null_str, |s| ChunkType::from(s).into())(data)?)
    }

    pub async fn write(
        &self,
        stream: &mut (dyn AsyncWrite + Send + Sync + Unpin),
    ) -> Result<'static, ()> {
        stream.write_all(<&str>::from(self.id).as_ref()).await?;
        stream.write_all(&[0]).await?;

        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, From)]
pub struct MServerInfo<'a> {
    pub name: &'a str,
}

impl<'a> MServerInfo<'a> {
    pub fn parse(data: &'a [u8]) -> Result<'a, (&'a [u8], Self)> {
        Ok(map(null_str, From::from)(data)?)
    }

    pub async fn write(
        &self,
        stream: &mut (dyn AsyncWrite + Send + Sync + Unpin),
    ) -> Result<'static, ()> {
        stream.write_all(self.name.as_ref()).await?;
        stream.write_all(&[0]).await?;

        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq, Clone, From)]
pub enum ChunkData<'a> {
    MChat(MChat<'a>),
    MPlayer(MPlayer<'a>),
    MRequest(MRequest<'a>),
    MServerInfo(MServerInfo<'a>),
    UUpdate(UUpdate<'a>),
    UAudioPlay(UAudioPlay<'a>),
    UActionCollision(UActionCollision<'a>),
    HHead(HHead),
    Eof,
    Unknown(&'a str, &'a [u8]),
}

impl<'a, 'b: 'a> TryFrom<&'b Chunk<'a>> for ChunkData<'a> {
    type Error = Error<'a>;

    fn try_from(c: &'b Chunk<'a>) -> Result<'b, Self> {
        Ok(match c.typ {
            ChunkType::MChat => MChat::parse(&c.data)?.1.into(),
            ChunkType::MPlayer => MPlayer::parse(&c.data)?.1.into(),
            ChunkType::MRequest => MRequest::parse(&c.data)?.1.into(),
            ChunkType::MServerInfo => MServerInfo::parse(&c.data)?.1.into(),
            ChunkType::UUpdate => UUpdate::from(&c.data as &[u8]).into(),
            ChunkType::UAudioPlay => UAudioPlay::from(&c.data as &[u8]).into(),
            ChunkType::UActionCollision => UActionCollision::from(&c.data as &[u8]).into(),
            ChunkType::HHead => HHead::parse(&c.data)?.1.into(),
            ChunkType::Eof => ChunkData::Eof,
            ChunkType::Unknown(s) => ChunkData::Unknown(s, &c.data),
        })
    }
}

impl<'a> From<ChunkData<'a>> for Chunk<'a> {
    fn from(d: ChunkData<'a>) -> Self {
        match d {
            ChunkData::UUpdate(uupdate) => Chunk {
                typ: ChunkType::UUpdate,
                data: Cow::Borrowed(uupdate.into()),
            },
            ChunkData::UAudioPlay(uupdate) => Chunk {
                typ: ChunkType::UAudioPlay,
                data: Cow::Borrowed(uupdate.into()),
            },
            ChunkData::UActionCollision(uupdate) => Chunk {
                typ: ChunkType::UActionCollision,
                data: Cow::Borrowed(uupdate.into()),
            },
            ChunkData::Eof => Chunk {
                typ: ChunkType::Eof,
                data: Cow::Borrowed(&[]),
            },
            ChunkData::Unknown(s, d) => Chunk {
                typ: ChunkType::Unknown(s),
                data: Cow::Borrowed(d),
            },
            d => {
                // all others need same boilerplate
                let mut data = Vec::new();
                let typ = match d {
                    ChunkData::MChat(chunk) => {
                        chunk.write(&mut data).now_or_never().unwrap().unwrap();
                        ChunkType::MChat
                    }
                    ChunkData::MRequest(chunk) => {
                        chunk.write(&mut data).now_or_never().unwrap().unwrap();
                        ChunkType::MRequest
                    }
                    ChunkData::MServerInfo(chunk) => {
                        chunk.write(&mut data).now_or_never().unwrap().unwrap();
                        ChunkType::MServerInfo
                    }
                    ChunkData::HHead(chunk) => {
                        chunk.write(&mut data).now_or_never().unwrap().unwrap();
                        ChunkType::HHead
                    }
                    ChunkData::MPlayer(chunk) => {
                        chunk.write(&mut data).now_or_never().unwrap().unwrap();
                        ChunkType::MPlayer
                    }
                    _ => unreachable!(),
                };
                Chunk {
                    typ,
                    data: Cow::Owned(data),
                }
            }
        }
    }
}

#[derive(From, PartialEq, Eq, Clone)]
pub struct Chunk<'a> {
    pub typ: ChunkType<'a>,
    pub data: Cow<'a, [u8]>,
}

impl<'a> From<(ChunkType<'a>, &'a [u8])> for Chunk<'a> {
    fn from((typ, data): (ChunkType<'a>, &'a [u8])) -> Self {
        Chunk {
            typ,
            data: Cow::Borrowed(data),
        }
    }
}

impl fmt::Debug for Chunk<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Ok(data) = ChunkData::try_from(self) {
            <_ as fmt::Debug>::fmt(&data, f)
        } else {
            f.debug_struct("Chunk")
                .field("typ", &self.typ)
                .field("data", &self.data)
                .finish()
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct Frame<'a> {
    pub raw_chunks: SmallVec<[Chunk<'a>; 2]>,
}

impl<'a> Frame<'a> {
    pub fn new() -> Self {
        Default::default()
    }

    pub async fn write(
        self,
        stream: &'_ mut (dyn AsyncWrite + Send + Sync + Unpin + '_),
    ) -> Result<'static, ()> {
        for chunk in &self.raw_chunks {
            let name: &str = chunk.typ.into();
            stream.write_all(name.as_ref()).await?;
            stream.write_all(&[0]).await?;
            let len = u32::try_from(chunk.data.len())
                .expect("chunk too large")
                .to_le_bytes();
            stream.write_all(&len).await?;
            stream.write_all(&chunk.data).await?;
        }
        stream.write_all(b"\r\n\0" as &[u8]).await?;
        stream.flush().await?;

        Ok(())
    }
}

pub fn boolean<'a, E>(data: &'a [u8]) -> IResult<&'a [u8], bool, E>
where
    E: ParseError<&'a [u8]>,
{
    map(le_u8, |n| n != 0)(data)
}

pub fn null_str<'a, E>(data: &'a [u8]) -> IResult<&'a [u8], &'a str, E>
where
    E: ParseError<&'a [u8]>,
{
    map_res(
        terminated(take_until(&[0u8] as &[u8]), tag(&[0u8] as &[u8])),
        str::from_utf8,
    )(data)
}

pub fn chunk_type<'a, E>(data: &'a [u8]) -> IResult<&'a [u8], ChunkType<'a>, E>
where
    E: ParseError<&'a [u8]>,
{
    map(null_str, From::from)(data)
}

pub fn chunk<'a, E>(data: &'a [u8]) -> IResult<&'a [u8], Chunk<'a>, E>
where
    E: ParseError<&'a [u8]>,
{
    let (data, chunk_type) = chunk_type(data)?;
    let (data, chunk_data) = if chunk_type != ChunkType::Eof {
        length_data(le_u32)(data)?
    } else {
        (data, &[] as &[u8])
    };
    Ok((data, (chunk_type, chunk_data).into()))
}

pub fn frame<'a, E>(data: &'a [u8]) -> IResult<&'a [u8], Frame<'a>, E>
where
    E: ParseError<&'a [u8]> + Clone,
{
    let mut it = iterator(data, chunk);
    let raw_chunks = it.take_while(|c| c.typ != ChunkType::Eof).collect();

    Ok((it.finish()?.0, Frame { raw_chunks }))
}

#[cfg(test)]
mod test {
    use super::*;
    use nom::error::VerboseError;
    use smallvec::smallvec;
    use std::convert::TryInto;

    #[test]
    fn parse_null_str() {
        assert_eq!(
            null_str::<VerboseError<_>>(b"hi\0\xFF").unwrap(),
            (&[0xFFu8] as &[u8], "hi")
        );
    }

    #[test]
    fn parse_chunk() {
        assert_eq!(
            chunk::<VerboseError<_>>(b"hi\0\x03\0\0\0\x01\x02\x03end").unwrap(),
            (
                b"end" as &[u8],
                Chunk {
                    typ: ChunkType::Unknown("hi".into()),
                    data: Cow::Borrowed(&[1, 2, 3]),
                }
            )
        )
    }

    #[test]
    fn parse_eof() {
        assert_eq!(
            chunk::<VerboseError<_>>(b"\r\n\0end").unwrap(),
            (
                b"end" as &[u8],
                Chunk {
                    typ: ChunkType::Eof,
                    data: Cow::Borrowed(&[]),
                }
            )
        )
    }

    #[test]
    fn parse_frame() {
        assert_eq!(
            frame::<VerboseError<_>>(b"hi\0\x03\0\0\0\x01\x02\x03bye\0\0\0\0\0\r\n\0end").unwrap(),
            (
                b"end" as &[u8],
                Frame {
                    raw_chunks: smallvec![
                        Chunk {
                            typ: ChunkType::Unknown("hi"),
                            data: Cow::Borrowed(&[1, 2, 3]),
                        },
                        Chunk {
                            typ: ChunkType::Unknown("bye"),
                            data: Cow::Borrowed(&[]),
                        }
                    ]
                }
            )
        )
    }

    #[test]
    fn parse_uupdate() {
        let frame = frame::<VerboseError<_>>(b"nU\0\x05\0\0\0\x01\0\0\0a\r\n\0")
            .unwrap()
            .1;
        let uupdate: ChunkData = (&frame.raw_chunks[0]).try_into().unwrap();
        let uupdate = if let ChunkData::UUpdate(uupdate) = uupdate {
            uupdate
        } else {
            panic!("not uupdate");
        };
        assert_eq!(uupdate.id(), 1);
        assert_eq!(&uupdate.remainder(), b"a");
        let data: &[u8] = uupdate.into();
        assert_eq!(data, &[1, 0, 0, 0, b'a']);
    }
}
