# tempfiles-rs

![Project badge](https://img.shields.io/badge/language-Rust-blue.svg)
![Crates.io License](https://img.shields.io/crates/l/tempfiles-rs)
![GitHub Release](https://img.shields.io/github/v/release/PlexSheep/tempfiles-rs)
![GitHub language count](https://img.shields.io/github/languages/count/PlexSheep/tempfiles-rs)
[![Rust CI](https://github.com/PlexSheep/tempfiles-rs/actions/workflows/cargo.yaml/badge.svg)](https://github.com/PlexSheep/hedu/actions/workflows/cargo.yaml)

Easy file hosting service

* [GitHub](https://github.com/PlexSheep/tempfiles-rs)
* [crates.io](https://crates.io/crates/tempfiles-rs)
* [docs.rs](https://docs.rs/crate/tempfiles-rs/)

## Uploading

```bash
curl -v -X POST http://localhost:8080/file -F "name=passwd" -F "file=@/etc/passwd"
```

## System deps

```bash
apt-get install libmagic-dev
```
