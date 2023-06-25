//! # Intro
//!
//! `ret2shell`: a feature-riched CTF challenge platform. This website contains the
//! implemention details of `ret2shell`, not the user manual.
//!
//! This documentation is generated from the source code of `ret2shell`.
//! If you want to contribute to this project, refer this.
//!
//! > **DO NOT USE THIS PROJECT AS A LIBRARY IN YOUR PROJECT.
//! ALL THE MEMBERS ARE PRIVATE AND WILL CHANGE WITHOUT NOTICE.**
//!

#![warn(missing_docs)]

mod audit;
mod bucket;
mod cache;
mod captcha;
mod checker;
mod config;
mod controller;
mod entity;
mod logging;
mod media;
mod migrator;
mod oauth;
mod queue;
mod service;
mod traffic;
mod utility;

/// Server entry.
#[tokio::main]
async fn main() {
    println!("Hello, world!");
}
