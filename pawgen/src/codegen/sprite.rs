use std::cell::RefMut;

use crate::schema;

use super::{generate_next_id, BlocksBuilder, ProjectAsset};

pub struct SpriteBuilder {
    project_ref: super::ProjectCell,
    idx: usize,
}

impl SpriteBuilder {
    pub(super) fn new(project_ref: super::ProjectCell, idx: usize) -> Self {
        Self { project_ref, idx }
    }

    /// Gets mutable reference to the underlaying sprite this builder
    /// manages.
    pub fn sprite_ref<'builder>(&'builder self) -> RefMut<'builder, schema::ProjectTarget> {
        RefMut::map(self.project_ref.borrow_mut(), |project| {
            project.targets.iter_mut().nth(self.idx).unwrap()
        })
    }

    pub fn add_costume(&self, asset: &ProjectAsset) {
        self.sprite_ref().costumes.push(asset.into())
    }

    pub fn set_default_costume(&self, costume: impl AsRef<str>) -> &Self {
        let mut sprite = self.sprite_ref();
        sprite.current_costume = sprite
            .costumes
            .iter()
            .position(|c| c.name == costume.as_ref())
            .unwrap_or(0) as _;
        self
    }

    pub fn make_variable(
        &self,
        name: impl AsRef<str>,
        default_value: schema::Value,
    ) -> schema::Value {
        let id = generate_next_id();
        self.sprite_ref().variables.insert(
            id.clone(),
            schema::Variable {
                display_name: name.as_ref().to_owned(),
                value: match default_value {
                    schema::Value::Number(value) => schema::VariableValue::Number(value),
                    schema::Value::Text(value) => schema::VariableValue::Text(value),
                    _ => schema::VariableValue::Number(0f64),
                },
            },
        );

        schema::Value::Variable(id.clone(), name.as_ref().to_owned())
    }

    pub fn set_stage(&self, is_stage: bool) -> &Self {
        let mut sprite = self.sprite_ref();
        sprite.is_stage = is_stage;
        sprite.layer_order = 0;
        self
    }

    pub fn set_volume(&self, volume: u32) -> &Self {
        self.sprite_ref().volume = volume;
        self
    }

    pub fn set_visible(&self, visible: bool) -> &Self {
        self.sprite_ref().visible = visible;
        self
    }

    pub fn set_position(&self, x: i32, y: i32) -> &Self {
        let mut sprite = self.sprite_ref();
        sprite.x = x;
        sprite.y = y;
        self
    }

    pub fn set_size(&self, size: u32) -> &Self {
        self.sprite_ref().size = size;
        self
    }

    pub fn set_direction(&self, direction: i32) -> &Self {
        self.sprite_ref().direction = direction;
        self
    }

    pub fn set_draggable(&self, draggable: bool) -> &Self {
        self.sprite_ref().draggable = draggable;
        self
    }

    pub fn set_rotation_style(&self, rotation_style: String) -> &Self {
        self.sprite_ref().rotation_style = rotation_style;
        self
    }

    pub fn blocks_builder<'builder>(&'builder self) -> BlocksBuilder<'builder> {
        BlocksBuilder::new(RefMut::map(self.sprite_ref(), |sprite| &mut sprite.blocks))
    }
}
