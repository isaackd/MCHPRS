use crate::blocks::{BlockDirection};
use mchprs_blocks::{BlockFace, BlockPos};

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum ActionResult {
    Success,
    Pass,
}

impl ActionResult {
    fn is_success(self) -> bool {
        self == ActionResult::Success
    }
}

pub struct UseOnBlockContext {
    pub block_pos: BlockPos,
    pub block_face: BlockFace,
    pub block_direction: BlockDirection,
    pub cursor_y: f32,
}
