# Command Runner

Command Runner is a Rust application used for sending user specified commands via an IO interface and processing the responses received
on that interface.

## Installation

Use `cargo` to install Command Runner.

```bash
cargo install command-runner
```

## Usage

The Command Runner takes one mandatory command line argument, a configuration file. The configuration file must be a valid JSON file. configuration
options are outlined in the Configuration section. Below is an example of how to invoke the Command Runner

```Rust
command-runner --config-file config.json
```


## Contributing

Pull requests are welcome. For major changes, please open an issue first
to discuss what you would like to change.

Please make sure to update tests as appropriate.

## License

[MIT](https://choosealicense.com/licenses/mit/)
