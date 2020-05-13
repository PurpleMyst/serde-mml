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

pub struct EscapedFormatter<W: Write> {
    output: W,
    error: Option<io::Error>,
}

impl<W: Write> fmt::Write for EscapedFormatter<W> {
    fn write_char(&mut self, ch: char) -> fmt::Result {
        let result = if ch.is_ascii_punctuation() {
            write!(self.output, "\\{}", ch)
        } else {
            write!(self.output, "{}", ch)
        };

        match result {
            Ok(()) => Ok(()),
            Err(error) => {
                self.error = Some(error);
                Err(fmt::Error)
            }
        }
    }

    fn write_str(&mut self, s: &str) -> fmt::Result {
        s.chars().try_for_each(|ch| self.write_char(ch))
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

    fn escaped<T: fmt::Display>(&mut self, value: T) -> io::Result<()> {
        use fmt::Write;
        let mut formatter = EscapedFormatter {
            output: &mut self.output,
            error: None,
        };
        match formatter.write_fmt(format_args!("{}", value)) {
            Ok(()) => Ok(()),
            Err(fmt::Error) => Err(formatter.error.unwrap()),
        }
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
        self.escaped(text)?;
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
