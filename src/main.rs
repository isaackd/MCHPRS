use std::path::Path;
use std::time::Instant;

use mchprs_core::blocks::{BlockPos, Block, BlockDirection};
use mchprs_core::plot::{PlotWorld, PLOT_WIDTH, data::empty_plot};
use mchprs_core::redpiler::{Compiler, CompilerOptions};
use mchprs_core::world::World;
use mchprs_core::world::storage::Chunk;
use mchprs_core::blocks::redstone::*;
use mchprs_save_data::plot_data::PlotData;

const START_BUTTON: BlockPos = BlockPos::new(0, 0, 0);

fn main() {
    mandelbrot_full();
}

fn load_world(path: impl AsRef<Path>) -> PlotWorld {
    // let data = PlotData::load_from_file(path).unwrap();
    let data = empty_plot();

    let chunks: Vec<Chunk> = data
        .chunk_data
        .into_iter()
        .enumerate()
        .map(|(i, c)| Chunk::load(i as i32 / PLOT_WIDTH, i as i32 % PLOT_WIDTH, c))
        .collect();

    println!("Chunks will be: {:?}", &chunks.len());

    let mut plot_world = PlotWorld {
        x: 0,
        z: 0,
        chunks,
        to_be_ticked: data.pending_ticks,
    };

    for x in 2..5 {
        let added_block = plot_world.set_block(BlockPos::new(x, 0, 0), Block::RedstoneWire {
            wire: RedstoneWire::new(
                RedstoneWireSide::None,
                RedstoneWireSide::None,
                RedstoneWireSide::None,
                RedstoneWireSide::None,
                0
            ),
        });

        println!("added redstone at {}: {}", x, added_block);
    }

    let added_block = plot_world.set_block(BlockPos::new(1, 0, 0), Block::RedstoneRepeater {
        repeater: RedstoneRepeater::new(3, BlockDirection::West, false, false),
    });

    let added_block = plot_world.set_block(BlockPos::new(0, 0, 0), Block::Lever { 
        lever: Lever::new(LeverFace::Floor, BlockDirection::North, false),
    });

    // let used = plot_world.get_block(BlockPos::new(1, 0, 0))
    //             .on_use(&mut plot_world, BlockPos::new(1, 0, 0), None);

    // println!("used repeater: {:?}", used);

    // let added_block = plot_world.set_block(BlockPos::new(0, 0, 0), Block::Stone {  });

    println!("We added the block? {}", added_block);

    plot_world
}

fn init_compiler() -> (PlotWorld, Compiler) {
    let mut world = load_world("./chungus_mandelbrot_plot");
    let mut compiler: Compiler = Default::default();

    // println!("lever_use result: {:?} {:?}", result, lever);

    // let options = CompilerOptions::parse("-O");
    let options = CompilerOptions::default();
    compiler.compile(&mut world, options, Vec::new());
    compiler.on_use_block(&mut world, START_BUTTON);

    (world, compiler)
}

fn mandelbrot_full() {
    println!("Running full chungus mandelbrot, this can take a while!");
    let (mut world, mut compiler) = init_compiler();
    let start = Instant::now();
    // let num2 = 12411975;
    let num2 = 5;
    for i in 0..num2 {
        // compiler.inspect(START_BUTTON);
        if (i % 10000 == 0) {
            println!("{}", i);
        }
        compiler.tick(&mut world);
    }
    println!("Mandelbrot benchmark completed in {:?}", start.elapsed());
}
