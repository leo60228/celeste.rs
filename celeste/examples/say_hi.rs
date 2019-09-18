use async_std::net::TcpStream;
use async_std::task;
use celeste::ghostnet::*;
use futures::io::BufWriter;
use futures::prelude::*;
use smallvec::*;
use std::convert::TryFrom;
use std::env;
use std::error::Error;

async fn say_hi() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    let addr = env::args().nth(1).unwrap();
    let conn = TcpStream::connect(addr).await?;
    let (mut read, write) = conn.split();
    let mut write = BufWriter::new(write);

    let mut buf = Vec::new();
    let frame_a = loop {
        let i = buf.len();
        buf.push(0);
        read.read(&mut buf[i..]).await?;
        if let Ok((_, frame)) = frame::<()>(&buf) {
            break frame;
        }
    };

    println!("{:#?}", frame_a);

    let id_chunk: ChunkData = MPlayer {
        name: "leobot".into(),
        idle: false,
        ..Default::default()
    }
    .into();
    let id_frame = Frame {
        raw_chunks: smallvec![id_chunk.into()],
    };

    id_frame.write(&mut write).await?;
    write.flush().await?;

    let mut buf = Vec::new();
    let frame_b = loop {
        let i = buf.len();
        buf.push(0);
        read.read(&mut buf[i..]).await?;
        if let Ok((_, frame)) = frame::<()>(&buf) {
            break frame;
        }
    };

    println!("{:#?}", frame_b);

    let mut buf = Vec::new();
    let frame_c = loop {
        let i = buf.len();
        buf.push(0);
        read.read(&mut buf[i..]).await?;
        if let Ok((_, frame)) = frame::<()>(&buf) {
            break frame;
        }
    };

    println!("{:#?}", frame_c);

    let mut buf = Vec::new();
    let frame_d = loop {
        let i = buf.len();
        buf.push(0);
        read.read(&mut buf[i..]).await?;
        if let Ok((_, frame)) = frame::<()>(&buf) {
            break frame;
        }
    };

    println!("{:#?}", frame_d);

    let mut buf = Vec::new();
    let frame_e = loop {
        let i = buf.len();
        buf.push(0);
        read.read(&mut buf[i..]).await?;
        if if let Ok((_, frame)) = frame::<()>(&buf) {
            if frame.raw_chunks.iter().next_back().unwrap().typ == ChunkType::HHead {
                break frame;
            } else {
                true
            }
        } else {
            false
        } {
            buf.clear();
        }
    };

    println!("{:#?}", frame_e);

    let chat_chunk: ChunkData = MChat {
        text: "hello, world",
        ..Default::default()
    }
    .into();
    let chat_frame = Frame {
        raw_chunks: smallvec![chat_chunk.into()],
    };
    println!("{:#?}", chat_frame);

    chat_frame.write(&mut write).await?;
    write.flush().await?;

    let mut buf = Vec::new();
    let frame_f = loop {
        let i = buf.len();
        buf.push(0);
        read.read(&mut buf[i..]).await?;
        if if let Ok((_, frame)) = frame::<()>(&buf) {
            if frame.raw_chunks.iter().any(|c| c.typ == ChunkType::MChat) {
                break frame;
            } else {
                true
            }
        } else {
            false
        } {
            buf.clear();
        }
    };

    println!("{:#?}", frame_f);

    Ok(())
}

fn main() {
    if let Err(err) = task::block_on(say_hi()) {
        println!("{}", err);
    }
}
