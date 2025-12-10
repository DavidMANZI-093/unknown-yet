use std::{
    error::Error,
    io::{self, BufWriter, Stdout, StdoutLock, Write},
    os::fd::AsFd,
    time::Duration,
};

use crate::sprites::{
    food::Food,
    snake::{Direction, Snake},
};

use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
    execute, queue, style, terminal,
};
use rustix::termios::{isatty, tcgetwinsize};
use tokio::sync::broadcast::Sender;

pub struct Canvas<'a> {
    pub f_buffer: Vec<char>,
    _stdout: Stdout,
    _handle: BufWriter<StdoutLock<'a>>,
    _mspf: u64,
    _tx: Sender<String>,
    _width: u16,
    _height: u16,
    pub _width_i: u16,
    pub _height_i: u16,
}

impl<'a> Canvas<'a> {
    pub fn new(
        stdout: Stdout,
        mut handle: BufWriter<StdoutLock<'a>>,
        tx: Sender<String>,
    ) -> Result<Self, Box<dyn Error>> {
        if let Some((x, y)) = _get_size(&stdout)
            && x >= 160
            && y >= 40
        {
            match tx.send(format!(
                "(unknown-yet) info: initializing canvas (width: {}, height: {}, fps: {})",
                160,
                40,
                (1000 / 90)
            )) {
                Ok(_) => {}
                Err(e) => eprintln!(
                    "(unknown-yet) Error: could not broadcast message over tx: {}",
                    e
                ),
            }
            Ok(Self {
                f_buffer: vec![' '; (x * y) as usize],
                _stdout: stdout,
                _handle: handle,
                _mspf: 90,
                _tx: tx,
                _width: x,
                _height: y,
                _width_i: 160,
                _height_i: 40,
            })
        } else {
            execute!(&mut handle, cursor::Show, terminal::LeaveAlternateScreen)?;
            terminal::disable_raw_mode()?;
            Err("terminal size too small".into())
        }
    }

    pub fn animate<F>(
        &mut self,
        _snake: &mut Snake,
        _food: &mut Food,
        mut update: F,
    ) -> io::Result<()>
    where
        F: FnMut(&mut Self, &mut Snake, &mut Food, u16, u16),
    {
        loop {
            update(self, _snake, _food, self._width_i, self._height_i);
            queue!(self._handle, cursor::MoveTo(0, 0))?;
            for y in 0..self._height_i {
                for x in 0..self._width_i {
                    let idx = (y * self._width_i) + x;
                    queue!(self._handle, style::Print(self.f_buffer[idx as usize]))?;
                }
                queue!(self._handle, style::Print("\r\n"))?;
            }
            self._handle.flush()?;

            std::thread::sleep(Duration::from_millis(self._mspf));

            if event::poll(Duration::from_millis(0))? {
                match event::read()? {
                    Event::Key(key_event) => match key_event.code {
                        KeyCode::Char('q') | KeyCode::Esc => break,
                        KeyCode::Up => _snake.turn(Direction::Up),
                        KeyCode::Down => _snake.turn(Direction::Down),
                        KeyCode::Left => _snake.turn(Direction::Left),
                        KeyCode::Right => _snake.turn(Direction::Right),

                        _ => {}
                    },
                    Event::Resize(w, h) => self._on_resize(w, h)?,
                    _ => {}
                }
            }
        }

        execute!(self._handle, cursor::Show, terminal::LeaveAlternateScreen)?;
        terminal::disable_raw_mode()?;
        Ok(())
    }

    pub fn _on_resize(&mut self, _w: u16, _h: u16) -> io::Result<()> {
        self._respawn(_w, _h)?;
        match self
            ._tx
            .send(String::from("(unknown-yet) resize event detected"))
        {
            Ok(_) => {}
            Err(e) => eprintln!(
                "(unknown-yet) Error: could not broadcast message over tx: {}",
                e
            ),
        }
        Ok(())
    }

    fn _respawn(&mut self, _w: u16, _h: u16) -> io::Result<()> {
        if let Some((x, y)) = _get_size(&self._stdout)
            && x >= self._width_i
            && y >= self._height_i
        {
            self._width = _w;
            self._height = _h;
            self.generic_buff();
            Ok(())
        } else {
            self.clean_up()?;
            panic!("terminal size too small");
        }
    }

    pub fn generic_buff(&mut self) {
        self.f_buffer = vec![' '; (self._width_i * self._height_i) as usize];
        for y in 0..self._height_i {
            for x in 0..self._width_i {
                let idx = (y * self._width_i) + x;
                let v_edge = x == 0 || x == self._width_i - 1;
                let h_edge = y == 0 || y == self._height_i - 1;

                if h_edge && v_edge {
                    self.f_buffer[idx as usize] = '+';
                } else if v_edge {
                    self.f_buffer[idx as usize] = '|';
                } else if h_edge {
                    self.f_buffer[idx as usize] = '~';
                }
            }
        }
    }

    pub fn clean_up(&mut self) -> io::Result<()> {
        execute!(self._handle, cursor::Show, terminal::LeaveAlternateScreen)?;
        terminal::disable_raw_mode()?;
        Ok(())
    }
}

fn _get_size(_stdout: &Stdout) -> Option<(u16, u16)> {
    if !isatty(_stdout.as_fd()) {
        return None;
    }

    let winsize = tcgetwinsize(_stdout).ok()?;

    let (x, y) = (winsize.ws_col, winsize.ws_row);

    if x > 0 && y > 0 { Some((x, y)) } else { None }
}
