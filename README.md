# 🦀 Mensa API - Studentenwerk Leipzig 🦀
A REST API which efficiently scrapes and caches Studentenwerk Leipzig meals.

Data is fully dynamic:
added or removed *🥦Mensae🥦* don't have to be added, supported locations are fetched dynamically.
# CampusUnbloat
While being fairly unopinionated, this API is part of the [**CampusUnbloat**](https://github.com/greybaron/campus-unbloat) project.

## Build Dependencies
* A working Rust toolchain
* SSL development files (e.g. `libssl-dev` on Debian)
* SQLite3 development files (e.g. `libsqlite3-dev` on Debian)

## Using the API
While there is no formal documentation yet, the API is very straightforward - have a look in `src/routes.rs`.
## Data policy
No data is ever logged or stored. It's not like it is particularly interesting anyways.
