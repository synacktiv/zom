# ZOM (Zed Offline Mirror)

_zom_ is a mirror server for [Zed code editor](https://zed.dev/).

## Features

- Mirror of zed extensions (<https://zed.dev/extensions>)
- Mirror of zed updates, providing an offline standalone installer.
- Efficient synchronisation of new extensions.

## Usage

Once deployed at `http://<ZOM_SERVER>`, you can use it in zed by adding the following in you configuration file:

```json
"server_url": "http://<ZOM_SERVER>",
```

You can also install zed from this server:

```bash
curl http://<ZOM_SERVER>/install.sh | sh
```

or you can go to `http://<ZOM_SERVER>`.

## Administration

_zom_ works in two steps:

1. Synchronisation of the mirror.
2. Serving the mirror on HTTP.

Step 1 creates a directory structure for the mirror by connecting to an
upstream. This directory is used by step 2 to serve the mirror.

``` bash
# create the mirror by connecting to an upstream server (by default zed.dev).
zom sync -d /path/to/mirror

# serve the mirror on localhost:8080
zom serve -d /path/to/mirror -l localhost:8080
```

See `zom sync --help` and `zom serve --help` for more options for these commands.

Also see `zom --help` for more options provided by the binary.

All options can also be provided using a _toml_ configuration file, as provided in this repository `config.toml`.

## Development

```bash
cargo run
```

## Packaging

### Debian

```bash
# install cargo deb <https://github.com/kornelski/cargo-deb>
# only required once
cargo install cargo-deb

# build the deb in target/debian
cargo deb
```
