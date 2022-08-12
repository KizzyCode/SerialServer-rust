[![License BSD-2-Clause](https://img.shields.io/badge/License-BSD--2--Clause-blue.svg)](https://opensource.org/licenses/BSD-2-Clause)
[![License MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![AppVeyor CI](https://ci.appveyor.com/api/projects/status/github/KizzyCode/serialServer-rust?svg=true)](https://ci.appveyor.com/project/KizzyCode/serialServer-rust)
[![docs.rs](https://docs.rs/serial_server/badge.svg)](https://docs.rs/serial_server)
[![crates.io](https://img.shields.io/crates/v/serial_server.svg)](https://crates.io/crates/serial_server)
[![Download numbers](https://img.shields.io/crates/d/serial_server.svg)](https://crates.io/crates/serial_server)
[![dependency status](https://deps.rs/crate/serial_server/0.1.0/status.svg)](https://deps.rs/crate/serial_server/0.1.0)


# `SerialServer`
Welcome to `SerialServer` 🎉

`SerialServer` is a tiny server that bridges a serial device to UDP - incoming packets are written to the serial
device's input, and the serial device's output can be forwarded to a given UDP address.


## Configuration
The server is configured via a config file. The path to the config file can be specified via:
 - the `SERIALSERVER_CONFIG` environment variable
 - the first command line argument

If no path is specified, the server expects a `config.toml` in the current working directory.


### Example configuration file
An example configuration file could look like this:

```toml
[serial]
# The path to the serial device
device = "/dev/tty.usbmodem21201"

# The baudrate of the serial connection (defaults to 115200)
baudrate = 115200


[udp]
# The UDP port to listen on for incoming packets
listen = "127.0.0.1:6666"

# The UDP port to send the serial device's output to (optional; if omitted, nothing is sent)
send = "224.0.0.1:6666"

# The TTL for outgoing UDP packets (defaults to 0)
ttl = 0


[log]
# Whether to log the serial device's I/O to stdout (defaults to false)
enabled = true
```

## Notes on security
This server acts as a simple, stupid bridge – there is *no* authentication or data validation. The primary usecase for
this server is to run within a docker container or similar with UDP on localhost as brigde to e.g. NodeRED.
