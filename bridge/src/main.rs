#![recursion_limit = "256"]

use async_std::net::TcpStream;
use async_std::task;
use celeste::ghostnet::*;
use futures::channel::mpsc::{self, UnboundedReceiver, UnboundedSender};
use futures::prelude::*;
use slice_deque::SliceDeque;
use smallvec::*;
use std::collections::HashMap;

use std::convert::TryFrom;

use futures::io::BufWriter;
use std::env;

use serenity::{cache::CacheRwLock, model::channel::Message, prelude::*};

type Result<'a, T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync + 'a>>; // 4

pub async fn ghostnet(
    sender: UnboundedSender<String>,
    mut receiver: UnboundedReceiver<String>,
) -> Result<'static, ()> {
    let mut id = None;

    let addr = env::args().nth(1).unwrap();

    let conn = TcpStream::connect(addr).await?;

    let (mut read, write) = (&conn, &conn);
    let mut write = BufWriter::new(write);

    let (response_tx, mut response_rx) = mpsc::unbounded::<Vec<u8>>();

    let send = async move {
        loop {
            futures::select! {
                response = response_rx.next() => {
                    let response = response.ok_or_else(|| {
                        std::io::Error::new(
                            std::io::ErrorKind::UnexpectedEof,
                            "couldn't get next response",
                        )
                    })?;
                    println!("got response, sending {:?}", response);
                    write.write_all(&response).await?;
                    write.flush().await?;
                },
                message = receiver.next() => {
                    let message = if let Some(msg) = message {
                        msg
                    } else {
                        continue;
                    };
                    let chat: ChunkData = MChat {
                        text: &message,
                        ..Default::default()
                    }.into();
                    let frame = Frame {
                        raw_chunks: smallvec![chat.into()],
                    };
                    frame.write(&mut write).await?;
                },
                complete => {
                    return Result::Err(std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "couldn't get next response").into())
                }
            }
        }

        #[allow(unreachable_code)]
        Result::Ok(())
    };

    let recv = async move {
        let mut buf: SliceDeque<u8> = SliceDeque::new();
        let mut start = 0;

        let mut users = HashMap::new();

        loop {
            let unparsed;

            match frame::<celeste::Error>(&buf[..start]) {
                Ok((rem, frame)) => {
                    unparsed = rem.len();
                    eprintln!("got frame: {:?}", frame);

                    let user_id = frame
                        .raw_chunks
                        .iter()
                        .filter(|c| c.typ == ChunkType::HHead)
                        .map(|c| {
                            if let ChunkData::HHead(HHead { id }) = ChunkData::try_from(c).unwrap()
                            {
                                id
                            } else {
                                unreachable!()
                            }
                        })
                        .next();
                    let mut user_name = None;

                    for chunk in frame.raw_chunks {
                        match ChunkData::try_from(&chunk) {
                            Ok(ChunkData::MChat(chat)) => {
                                println!("got mchat");
                                if user_id == id {
                                    continue;
                                }

                                println!("forwarding mchat");
                                sender.unbounded_send(format!(
                                    "<{}> {}",
                                    users
                                        .get(&user_id.unwrap())
                                        .map(<_>::as_ref)
                                        .unwrap_or("server"),
                                    chat.text
                                ))?;
                                println!("forwarded mchat");
                            }
                            Ok(ChunkData::MRequest(req)) => {
                                println!("got request for {:?}", req.id);
                                if req.id == ChunkType::MPlayer {
                                    println!("responding");
                                    let player = ChunkData::MPlayer(MPlayer {
                                        name: "leobot".into(),
                                        ..Default::default()
                                    });
                                    let frame = Frame {
                                        raw_chunks: smallvec![player.into()],
                                    };
                                    println!("responding with {:#?}", frame);
                                    let mut buf = Vec::new();
                                    frame.write(&mut buf).now_or_never().unwrap()?;
                                    println!("wrote response");
                                    response_tx.unbounded_send(buf)?;
                                    println!("sent response");
                                }
                            }
                            Ok(ChunkData::MPlayer(player)) => {
                                user_name = Some(player.name.to_string());
                            }
                            Ok(ChunkData::MServerInfo(_)) => {
                                id = Some(user_id.unwrap());
                            }
                            Ok(ChunkData::Unknown(ty, _)) => {
                                println!("unknown chunk {:?}", ty);
                            }
                            _ => continue,
                        }
                    }

                    if let (Some(id), Some(name)) = (user_id, user_name) {
                        users.insert(id, name);
                    }
                }
                res @ Err(nom::Err::Incomplete(_)) => {
                    std::mem::drop(res);

                    buf.extend(std::iter::repeat(0).take(start + 128 - buf.len()));
                    eprintln!("reading");
                    let read = read.read(&mut buf[start..]).await?;
                    if read != 0 {
                        start += read;
                    } else {
                        return Result::Err(
                            std::io::Error::new(
                                std::io::ErrorKind::UnexpectedEof,
                                "couldn't read frame",
                            )
                            .into(),
                        );
                    }
                    continue;
                }
                Err(_) => {
                    return Result::Err(
                        std::io::Error::new(std::io::ErrorKind::InvalidData, "bad chunk").into(),
                    )
                }
            }

            let parsed = start - unparsed;
            start -= parsed;
            for _ in 0..parsed {
                buf.pop_front();
            }
        }

        #[allow(unreachable_code)]
        Result::Ok(())
    };

    println!("handle done");
    if let Err(err) = future::try_join(send, recv).await {
        eprintln!("error handling socket: {}", err);
    }

    Ok(())
}

#[allow(clippy::unreadable_literal)]
const CHANNEL_ID: u64 = 622469967513780274;

struct Handler(pub UnboundedSender<String>);

impl EventHandler for Handler {
    fn message(&self, ctx: Context, msg: Message) {
        if msg.author.id == ctx.http.get_current_user().unwrap().id {
            return;
        }

        if msg.channel_id == CHANNEL_ID {
            self.0
                .unbounded_send(format!(
                    "<{}> {}",
                    msg.author_nick(&ctx)
                        .unwrap_or_else(|| msg.author.name.clone()),
                    msg.content_safe(&ctx)
                ))
                .unwrap();
        }
    }
}

fn main() {
    let (discord_tx, discord_rx) = mpsc::unbounded();
    let (ghostnet_tx, mut ghostnet_rx) = mpsc::unbounded();

    let token = env::var("DISCORD_TOKEN").unwrap();
    let mut client = Client::new(&token, Handler(discord_tx)).unwrap();

    let channel = client
        .cache_and_http
        .http
        .get_channel(CHANNEL_ID)
        .unwrap()
        .guild()
        .unwrap();
    let http = client.cache_and_http.clone();

    std::thread::spawn(move || client.start().unwrap());

    task::spawn(ghostnet(ghostnet_tx, discord_rx));

    futures::executor::block_on(async move {
        loop {
            let msg = ghostnet_rx.next().await.unwrap();
            let safe_msg =
                serenity::utils::content_safe(http.cache.clone(), &msg, &Default::default());
            channel.read().say(&http.http, safe_msg).unwrap();
        }
    });
}
