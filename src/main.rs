mod network;
//mod permissions;
mod blocks;
mod items;
mod player;
mod plot;
mod server;

#[macro_use]
extern crate bitflags;

use server::MinecraftServer;

fn main() {
    MinecraftServer::run();
}
