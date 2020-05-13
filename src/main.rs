use std::io;

use serde::Serialize;

use serde_md::ser::Serializer;

fn main() {
    let mut serializer = Serializer::new(io::stdout());
    (0u8..=3).serialize(&mut serializer).unwrap();
}
