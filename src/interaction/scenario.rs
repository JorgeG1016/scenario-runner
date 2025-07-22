use serde::Deserialize;

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum CommandType {
    Normal {
        send: String,
        expect_prefix: String,
        expect_exact: String,
        timeout: u64,
        delay: u64,
        description: Option<String>,
    },
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawCommand {
    #[serde(default)]
    send: Option<String>,
    expect_prefix: String,
    expect_exact: String,
    timeout: u64,
    delay: u64,
    #[serde(default)]
    description: Option<String>,
}

#[derive(Debug, PartialEq)]
struct Command {
    send: Option<String>,
    expect_prefix: String,
    expect_exact: String,
    timeout: u64,
    delay: u64,
    description: Option<String>,
}
