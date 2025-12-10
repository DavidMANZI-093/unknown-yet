use core::renderer::Canvas;
use std::{
    io::{self, BufWriter, stdout},
    thread,
};

use crossterm::{cursor, execute, terminal};
use tokio::sync::broadcast::{self, Receiver, Sender, error::RecvError};
use zbus::{connection, interface};

use crate::sprites::{food::Food, snake::Snake};

mod core;
mod sprites;

// const WIDTH: u16 = 160;
// const HEIGHT: u16 = 40;

#[derive(Debug, Default, Clone)]
struct Logger;

#[interface(name = "org.zbus.UnknownYet0")]
impl Logger {}

fn main() -> io::Result<()> {
    let (tx, mut rx) = broadcast::channel(1024) as (Sender<String>, Receiver<String>);

    thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap();

        rt.block_on(async {
            // THEAD 2
            let logger = Logger;
            let _conn = connection::Builder::session()
                .unwrap()
                .name("org.zbus.UnknownYet")
                .unwrap()
                .serve_at("/org/zbus/UnknownYet", logger.clone())
                .unwrap()
                .build()
                .await
                .unwrap();

            loop {
                match rx.recv().await {
                    Ok(msg) => {
                        let _ = _conn
                            .emit_signal(
                                Option::<&str>::None,
                                "/org/zbus/UnknownYet",
                                "org.zbus.UnknownYet0",
                                "LogEvent",
                                &msg,
                            )
                            .await;
                    }
                    Err(RecvError::Lagged(_)) => {
                        let _ = _conn
                            .emit_signal(
                                Option::<&str>::None,
                                "/org/zbus/UnknownYet",
                                "org.zbus.UnknownYet0",
                                "LogEvent",
                                &"lagged",
                            )
                            .await;
                    }
                    Err(e) => {
                        let _ = _conn
                            .emit_signal(
                                Option::<&str>::None,
                                "/org/zbus/UnknownYet",
                                "org.zbus.UnknownYet0",
                                "LogEvent",
                                &format!("error: {}", e),
                            )
                            .await;
                    }
                }
            }
        });
    });

    match tx.send(String::from(
        "(unknown-yet) info: initialized main thread (thread 1 - game loop)",
    )) {
        Ok(_) => {}
        Err(e) => eprintln!(
            "(unknown-yet) Error: could not broadcast message over tx: {}",
            e
        ),
    }

    match tx.send(String::from(
        "(unknown-yet) info: initialized second thread (thread 2 - zbus)",
    )) {
        Ok(_) => {}
        Err(e) => eprintln!(
            "(unknown-yet) Error: could not broadcast message over tx: {}",
            e
        ),
    }

    terminal::enable_raw_mode()?;

    let stdout = stdout();
    let lock = stdout.lock();
    let mut handle = BufWriter::new(lock);
    execute!(&mut handle, terminal::EnterAlternateScreen, cursor::Hide)?;

    let mut canvas = match Canvas::new(stdout, handle, tx.clone()) {
        Ok(canvas) => {
            match tx.send(String::from("(unknown-yet) info: canvas created")) {
                Ok(_) => {}
                Err(e) => eprintln!(
                    "(unknown-yet) Error: could not broadcast message over tx: {}",
                    e
                ),
            }
            canvas
        }
        Err(e) => {
            panic!("problem creating canvas: {}", e);
        }
    };

    let mut food = Food::new(4, canvas._width_i, canvas._height_i);
    let mut snake = Snake::new(4, 8, 4, tx.clone());

    match tx.send(String::from("(unknown-yet) info: snake created")) {
        Ok(_) => {}
        Err(e) => eprintln!(
            "(unknown-yet) Error: could not broadcast message over tx: {}",
            e
        ),
    }

    canvas.generic_buff();
    match canvas.animate(
        &mut snake,
        &mut food,
        |canvas, _snake, _food, width, _height| {
            canvas.generic_buff();

            _food.eggs.iter().for_each(|egg| {
                let idx = (egg.y * width) + egg.x;
                if idx < canvas.f_buffer.len() as u16 {
                    canvas.f_buffer[idx as usize] = '@';
                }
            });

            _snake.segments.iter().for_each(|segment| {
                let idx = (segment.y * width) + segment.x;
                if idx < canvas.f_buffer.len() as u16 {
                    canvas.f_buffer[idx as usize] = 'o';
                }
            });

            _snake.forward(_food, canvas._width_i, canvas._height_i);
        },
    ) {
        Ok(_) => {}
        Err(e) => {
            canvas.clean_up()?;
            panic!("error: {}", e);
        }
    }

    Ok(())
}
