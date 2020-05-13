use std::fmt;
use std::io::{self, prelude::*};

/// How many spaces do we indent with?
pub const INDENT: usize = 4;

/// Handles writing Markdown to a `Write` object
pub struct Writer<W> {
    output: W,
}

#[derive(Debug, Clone, Copy)]
enum Bullet {
    DottedNumber(usize),
    Asterisk,
}

impl fmt::Display for Bullet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Bullet::DottedNumber(n) => write!(f, "{}.", n),
            Bullet::Asterisk => f.pad("*"),
        }
    }
}

impl Bullet {
    fn advance(&mut self) {
        match self {
            Bullet::DottedNumber(n) => *n += 1,
            Bullet::Asterisk => {}
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct List {
    depth: usize,
    bullet: Bullet,
}

impl<W: Write> Writer<W> {
    pub fn new(output: W) -> Self {
        Self { output }
    }

    #[must_use]
    pub fn ordered_list(&mut self, mut parent: Option<&mut List>) -> io::Result<List> {
        if let Some(parent) = &mut parent {
            self.bullet(Some(parent))?;
            writeln!(self.output)?;
        }

        Ok(List {
            depth: parent.map_or(0, |parent| parent.depth + 1),
            bullet: Bullet::DottedNumber(0),
        })
    }

    #[must_use]
    pub fn unordered_list(&mut self, mut parent: Option<&mut List>) -> io::Result<List> {
        if let Some(parent) = &mut parent {
            self.bullet(Some(parent))?;
            writeln!(self.output)?;
        }

        Ok(List {
            depth: parent.map_or(0, |parent| parent.depth + 1),
            bullet: Bullet::Asterisk,
        })
    }

    // XXX: We might be able to write this zero-alloc style if we use `std::fmt::Write`
    fn escaped<S: IntoIterator<Item = char>>(&mut self, s: S) -> io::Result<()> {
        s.into_iter().try_for_each(|ch| {
            if ch.is_ascii_punctuation() {
                write!(self.output, "\\{}", ch)
            } else {
                write!(self.output, "{}", ch)
            }
        })
    }

    fn bullet(&mut self, list: Option<&mut List>) -> io::Result<()> {
        if let Some(List { depth, bullet }) = list {
            write!(
                self.output,
                "{:indent$}{} ",
                "",
                bullet,
                indent = INDENT * *depth,
            )?;
            bullet.advance();
        }
        Ok(())
    }

    pub fn link<Text: fmt::Display, URI: fmt::Display>(
        &mut self,
        list: Option<&mut List>,
        text: Text,
        uri: URI,
    ) -> io::Result<()> {
        self.bullet(list)?;
        write!(self.output, "[")?;
        self.escaped(format!("{}", text).chars())?;
        writeln!(self.output, "]({})", uri)?;
        Ok(())
    }

    pub fn bytes_link<URI: fmt::Display>(
        &mut self,
        list: Option<&mut List>,
        buf: &[u8],
        uri: URI,
    ) -> io::Result<()> {
        self.bullet(list)?;
        write!(self.output, "[")?;

        {
            // This new scope brought to you by borrowck
            let mut encoder = base64::write::EncoderWriter::new(
                &mut self.output,
                base64::Config::new(base64::CharacterSet::UrlSafe, true),
            );
            encoder.write(buf)?;
            encoder.finish()?;
        }

        writeln!(self.output, "]({})", uri)?;
        Ok(())
    }
}
