# Mapserver backend

An [actix-web](https://actix.rs/) based server written in Rust for collaborative point annotations of image-based maps.

## Building

Requires Rust and `cargo` installed.

```bash
# install diesel_cli and execute migrations
cargo install diesel_cli --no-default-features --features sqlite-bundled
mkdir mock
diesel migration run --database-url mock/map.db

# add maps
mkdir mock/maps
# find a good map and use Zoomify free to generate the image tiles

# add webpage
# cp -r web/ mock/www

# build and run as usual
cargo build
cargo run # might require sudo permissions to bind to network
```

## Setup

The server expects certain data in paths specified by environment variables, which can be specified in a `.env` file at the server's working directory:

- `MAPSERVER_SERVER_ADDR` server listening IP address and port, e.g. '0.0.0.0:80' to listen to any IP at port 80.
- `MAPSERVER_DATABASE_URL` path/url to the [SQLite](https://sqlite.org/index.html) database generated using the [diesel-cli](https://lib.rs/crates/diesel_cli) with the provided migrations file (under `migrations/`).
- `MAPSERVER_WWW_PATH` path to the static webpage to serve. Note that all files under this path will be accessible.
- `MAPSERVER_DATA_PATH` path to necessary data for the server, see `Data` section.

### Data

Currently, under `MAPSERVER_DATA_PATH` only a folder `maps` is expected that contains the maps in [Zoomify](http://www.zoomify.com/free.htm) format.

## DB info

There are two tables, `points` and `maps`:

- `maps`: every entry in this table is a new map although not necessarily using different images.
    + `id`: the primary key.
    + `keystr`: a string identifier that will be used to generate the URL of the map. The URL to access a map is `{domain}/map/{keystr}`, e.g. `http://localhost/map/testmap` will retrieve the entry with `keystr` equal to `testmap`. Entries with collisions on this attribute will get shadowed by the one with lower `id` (most likely).
    + `path`: where the Zoomify-based map folder is located under `MAPSERVER_DATA_PATH/maps`.
- `points`: each of the annotations that the maps can have.
    + `id`: the primary key.
    + `mapid`: the map this point belongs to.
    + `coordx`: the floating-point x-coordinate of the point.
    + `coordy`: the floating-point y-coordinate of the point.
    + `title`: the title of the point, can be null.
    + `body`: the description of the point, can be null.

# Disclaimer

This is a very early-stage pet project. It can (and will) contain critical security issues.