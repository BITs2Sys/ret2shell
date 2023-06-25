//! This module loads the configuration from the file and provided them to other modules.
//!
//! # Configuration file
//!
//! We use `toml` crate to parse configuration file.
//!
//! The configuration file will be looked up in the following order:
//!
//! 1. `./config/config.toml`: the current working directory
//! 2. `~/.config/ret2shell/config.toml`: the user's XDG config directory
//! 3. `/etc/ret2shell/config.toml`: the system config directory
//!
//! `ret2shell` server will try them in order and use the first one it found.
//!
//! # Management
//!
//! In previous Cyber Terminal implementions, the config file could be modified on-the-fly
//! and the server will reload the configuration automatically. This affects the ability to
//! implement cluster deployment and load balancing on the server, so we removed this
//! feature on `ret2shell`. The configuration file will be readonly after the server started.
//!
//! If you want to change the configuration, you should manually edit it through DevOps tools
//! then restart the server.
//!
//! For convinience, we move some configurations into the database, so that you can still
//! change them through the web interface.
//!
