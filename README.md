# Scenario Runner

The Scenatio Runner is a Rust application used for integration testing. Users specify *scenarios* that contains commands to be sent on a user
configured connection. Commands are sent by the application and data received on the connection is processed and checked to see if any of it
matches what the user specified.

In the end, the application will output results and log files outlining if the command sent passed or failed and other relevant information.

## Installation

Use `cargo` to install Scenario Runner.

```bash
cargo install scenario-runner
```

## Usage

The following is how to invoke the Scenario Runner from the command line:

```bash
scenario-runner
```

The Scenario-Runner takes the following command line arguments

- `--help`, `-h`: Argument that displays how to run the Command Runner and it's supported arguments, basically what this section of the `README.md` is
- `--config-file`, `-c`: An optional argument that allows the user to specify a configuration file, defaults to `./config.json`
- `--version`, `-V`: Argument that displays the version of the application

## Contributing

Pull requests are welcome. For major changes, please open an issue first
to discuss what you would like to change.

Please make sure to update tests as appropriate.

## License

[MIT](https://choosealicense.com/licenses/mit/)
