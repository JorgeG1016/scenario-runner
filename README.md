# Command Runner

The Command Runner is a Rust application used for sending user specified commands via an IO connection and processing the responses received
on that connection.

## Installation

Use `cargo` to install Command Runner.

```bash
cargo install command-runner
```

## Usage

The following is how to invoke the Command Runner from the command line:

```bash
command-runner
```

The Command-Runner takes the following command line arguments

- `--help`, `-h`: Argument that displays how to run the Command Runner and it's supported arguments, basically what this section of the `README.md` is
- `--config-file`, `-c`: An optional argument that allows the user to specify a configuration file, defaults to `./config.json`
- `--version`, `-V`: Argument that displays the version of the application

## Contributing

Pull requests are welcome. For major changes, please open an issue first
to discuss what you would like to change.

Please make sure to update tests as appropriate.

## License

[MIT](https://choosealicense.com/licenses/mit/)
