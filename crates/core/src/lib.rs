#![deny(rust_2018_idioms)]
#![feature(min_specialization, once_cell)]

#[macro_use]
mod utils;
pub mod blocks;
pub mod plot;
pub mod redpiler;
pub mod world;
pub mod items;

#[macro_use]
extern crate bitflags;
