use std::sync::Arc;

use super::{BlockDefinitions, Sprite};

#[derive(Debug)]
pub struct Project {
    pub(super) block_definitions: Arc<BlockDefinitions>,
    pub(super) sprites: Vec<Sprite>,
}

impl Project {
    pub fn new() -> Self {
        Self {
            block_definitions: BlockDefinitions::new(),
            sprites: Vec::new(),
        }
    }

    pub fn get_definitions(&self) -> &BlockDefinitions {
        &self.block_definitions
    }

    pub fn add_sprite(&mut self, sprite: Sprite) {
        self.sprites.push(sprite)
    }
}
