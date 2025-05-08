<div align="center">
    <img alt="icon" src="./data/static/img/logo.svg" width="60%"/>
    <h3>🗃️ tempfiles-rs ⏰</h3>
    <p>
        Easy file hosting and sharing service
    </p>
    <br/>
    <a href="https://github.com/PlexSheep/tempfiles-rs/actions/workflows/release.yaml">
        <img src="https://img.shields.io/github/actions/workflow/status/PlexSheep/tempfiles-rs/release.yaml?label=Release" alt="Release Status"/>
    </a>
    <a href="https://github.com/PlexSheep/tempfiles-rs/actions/workflows/cargo.yaml">
        <img src="https://img.shields.io/github/actions/workflow/status/PlexSheep/tempfiles-rs/cargo.yaml?label=Rust%20CI" alt="Rust CI"/>
    </a>
    <a href="https://github.com/PlexSheep/tempfiles-rs/blob/master/LICENSE">
        <img src="https://img.shields.io/github/license/PlexSheep/tempfiles-rs" alt="License"/>
    </a>
    <a href="https://github.com/PlexSheep/tempfiles-rs/releases">
        <img src="https://img.shields.io/github/v/release/PlexSheep/tempfiles-rs" alt="Release"/>
    </a>
    <br/>
    <a href="https://rust-lang.org">
        <img src="https://img.shields.io/badge/language-Rust-blue.svg" alt="Rust"/>
    </a>
    <a href="https://crates.io/crates/tempfiles-rs">
        <img alt="Crates.io MSRV" src="https://img.shields.io/crates/msrv/tempfiles-rs">
        <img alt="Crates.io Total Downloads" src="https://img.shields.io/crates/d/tempfiles-rs">
    </a>
    <a href="https://docs.rs/tempfiles-rs/latest/tempfiles-rs">
    <img alt="docs.rs" src="https://img.shields.io/docsrs/tempfiles-rs">
    </a>
</div>

# tempfiles-rs

![Project badge](https://img.shields.io/badge/language-Rust-blue.svg)
![Crates.io License](https://img.shields.io/crates/l/tempfiles-rs)
![GitHub Release](https://img.shields.io/github/v/release/PlexSheep/tempfiles-rs)
![GitHub language count](https://img.shields.io/github/languages/count/PlexSheep/tempfiles-rs)
[![Rust CI](https://github.com/PlexSheep/tempfiles-rs/actions/workflows/cargo.yaml/badge.svg)](https://github.com/PlexSheep/hedu/actions/workflows/cargo.yaml)

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
