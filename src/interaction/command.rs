use serde::Deserialize;

#[derive(Deserialize)]
#[serde(tag = "type")]
enum RawSendable {
    Hex {send: String},
    Text {send: String},

}

#[derive(Deserialize)]
#[serde(tag = "type")]
enum RawType {
    Standard {
        send: Option<RawSendable>,
        expect_prefix: String,
        expect_exact: String,
        timeout: u64,
        delay: u64,
    },
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawCommand {
    command: RawType,
    #[serde(default)]
    description: Option<String>,
}

pub enum Sendable {
    Hex {send: Vec<u8>},
    Text {send: Vec<u8>}
}

pub enum Type {
    Standard {
        send: Option<Sendable>,
        expect_prefix: Vec<u8>,
        expect_exact: Vec<u8>,
        timeout: u64,
        delay: u64
    }
}

struct Command {
    command: Type,
    description: Option<String>
}