use std::io;

use serde::Serialize;

use serde_mml::ser::Serializer;

fn main() {
    let mut serializer = Serializer::new(io::stdout());

    "/mnt/c/Users".serialize(&mut serializer).unwrap();
}
