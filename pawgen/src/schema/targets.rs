use std::collections::HashMap;

use serde::ser::SerializeTuple;

use super::ProjectBlocks;

/// Targets is a structure that holds all the sprites,
/// stage, and other stuff like that. This is also
/// where the code belongs.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectTarget {
    pub is_stage: bool,
    pub name: String,
    /// Map of variable ids to their names and values.
    pub variables: HashMap<String, Variable>,
    pub lists: HashMap<String, ()>,
    pub broadcasts: HashMap<String, ()>,
    /// Code for the sprite.
    pub blocks: ProjectBlocks,
    pub costumes: Vec<Costume>,
    pub current_costume: u32,
    pub sounds: Vec<Sound>,
    pub volume: u32,
    pub layer_order: u32,
    pub visible: bool,
    pub x: i32,
    pub y: i32,
    pub size: u32,
    pub direction: i32,
    pub draggable: bool,
    pub rotation_style: String,
}

/// Scratch variable tuple. This consists of variable name and default value.
#[derive(Debug, Clone)]
pub struct Variable {
    pub display_name: String,
    pub value: VariableValue,
}

/// Enum to represent multiple variable types that are possible in scratch.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum VariableValue {
    Number(f64),
    Text(String),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Costume {
    pub name: String,
    pub bitmap_resolution: u32,
    pub data_format: String,
    pub asset_id: String,
    pub md5ext: String,
    pub rotation_center_x: usize,
    pub rotation_center_y: usize,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Sound {
    pub name: String,
    pub asset_id: String,
    pub data_format: String,
    pub format: String,
    pub rate: u32,
    pub sample_count: u32,
    pub md5ext: String,
}

impl serde::Serialize for Variable {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut tuple = serializer.serialize_tuple(2)?;
        tuple.serialize_element(&self.display_name)?;
        tuple.serialize_element(&self.value)?;
        tuple.end()
    }
}

impl<'de> serde::Deserialize<'de> for Variable {
    fn deserialize<D>(_deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        todo!("Deserializer for 'Variable' is not yet implemented")
    }
}
