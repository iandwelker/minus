// This module contains the OutStream enum and it's related convenience functions

use std::io::{self, Stdout, Write};

use crossterm::tty::IsTty;

pub(crate) enum OutStream {
    SOut(Stdout),
    Vector(Vec<u8>),
}

impl OutStream {
    pub(crate) fn is_tty(&self) -> bool {
        match self {
            Self::SOut(s) => s.is_tty(),
            Self::Vector(_) => false,
        }
    }
}

impl Write for OutStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self {
            Self::SOut(s) => s.write(buf),
            Self::Vector(v) => {
                v.write_all(buf);
                Ok(buf.len())
            }
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self {
            Self::SOut(s) => s.flush(),
            Self::Vector(v) => Ok(()),
        }
    }
}

impl From<Stdout> for OutStream {
    fn from(s: Stdout) -> Self {
        Self::SOut(s)
    }
}
