[package]
name = "nexum-desktop"
version = "0.1.0"
edition = "2024"
authors = ["Nexum Contributors"]
description = "Nexum desktop application"
license = "MIT"

[dependencies]
tauri = { version = "2", features = [] }
tauri-plugin-shell = "2"

[build-dependencies]
tauri-build = { version = "2", features = [] }
