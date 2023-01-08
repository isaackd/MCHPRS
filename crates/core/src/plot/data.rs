use super::{Chunk, PlotWorld, PLOT_WIDTH, PLOT_BLOCK_WIDTH};
use anyhow::{Context, Result};
use mchprs_save_data::plot_data::{ChunkData, PlotData, Tps};
use std::path::Path;
use std::sync::LazyLock;
use std::time::Duration;

// TODO: where to put this?
pub fn sleep_time_for_tps(tps: Tps) -> Duration {
    match tps {
        Tps::Limited(tps) => {
            if tps > 10 {
                Duration::from_micros(1_000_000 / tps as u64)
            } else {
                Duration::from_millis(50)
            }
        }
        Tps::Unlimited => Duration::ZERO,
    }
}

pub fn load_plot(path: impl AsRef<Path>) -> Result<PlotData> {
    let path = path.as_ref();
    if path.exists() {
        Ok(PlotData::load_from_file(path)
            .with_context(|| format!("error loading plot save file at {}", path.display()))?)
    } else {
        Ok(EMPTY_PLOT.clone())
    }
}

pub fn empty_plot() -> PlotData {
    EMPTY_PLOT.clone()
}

fn generate_chunk(layers: i32, x: i32, z: i32) -> Chunk {
    let mut chunk = Chunk::empty(x, z);

    for ry in 0..layers {
        for rx in 0..16 {
            for rz in 0..16 {
                let block_x = (x << 4) | rx;
                let block_z = (z << 4) | rz;

                if block_x % PLOT_BLOCK_WIDTH == 0
                    || block_z % PLOT_BLOCK_WIDTH == 0
                    || (block_x + 1) % PLOT_BLOCK_WIDTH == 0
                    || (block_z + 1) % PLOT_BLOCK_WIDTH == 0
                {
                    chunk.set_block(rx as u32, ry as u32, rz as u32, 4564); // Stone Bricks
                } else {
                    chunk.set_block(rx as u32, ry as u32, rz as u32, 278); // Sandstone
                }
            }
        }
    }
    chunk
}

static EMPTY_PLOT: LazyLock<PlotData> = LazyLock::new(|| {
    let template_path = Path::new("./world/plots/pTEMPLATE");
    if template_path.exists() {
        PlotData::load_from_file(template_path).expect("failed to read template plot")
    } else {
        let mut chunks = Vec::new();
        for chunk_x in 0..PLOT_WIDTH {
            for chunk_z in 0..PLOT_WIDTH {
                chunks.push(generate_chunk(8, chunk_x, chunk_z));
            }
        }
        let mut world = PlotWorld {
            x: 0,
            z: 0,
            chunks,
            to_be_ticked: Vec::new(),
        };
        let chunk_data: Vec<ChunkData> = world.chunks.iter_mut().map(|c| c.save()).collect();
        PlotData {
            tps: Tps::Limited(10),
            chunk_data,
            pending_ticks: Vec::new(),
        }
    }
});
