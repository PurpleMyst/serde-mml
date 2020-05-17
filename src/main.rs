use std::io::{self};

fn main() {
    let mut deserializer = serde_json::Deserializer::from_reader(io::stdin());
    let mut serializer = serde_mml::ser::Serializer::new(io::stdout());
    serde_transcode::transcode(&mut deserializer, &mut serializer).unwrap();
}
