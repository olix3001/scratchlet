use std::collections::HashMap;

use serde::ser::{SerializeSeq, SerializeStruct, SerializeTuple};

#[derive(Debug, Clone, Default)]
pub struct ProjectBlocks {
    pub blocks: HashMap<String, Block>,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Block {
    pub opcode: String,
    pub next: Option<String>,
    pub parent: Option<String>,
    /// Inputs are user-provided data like number input or
    /// boolean condition.
    pub inputs: HashMap<String, BlockInput>,
    /// Fields are either choices or hidden data.
    pub fields: HashMap<String, BlockField>,
    pub shadow: bool,
    pub top_level: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub y: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mutation: Option<BlockMutation>,
}

#[derive(Debug, Clone)]
pub struct BlockInput {
    pub kind: usize,
    pub values: Vec<Value>,
}

impl serde::Serialize for BlockInput {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.values.len() + 1))?;
        seq.serialize_element(&self.kind)?;
        for value in self.values.iter() {
            seq.serialize_element(value)?;
        }
        seq.end()
    }
}

impl<'de> serde::Deserialize<'de> for BlockInput {
    fn deserialize<D>(_deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        todo!("Deserializer for BlockInput is not implemented")
    }
}

#[derive(Debug, Clone)]
pub enum BlockField {
    Variable(String, String),
    Argument(String),
}

impl serde::Serialize for BlockField {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match &self {
            Self::Variable(id, name) => {
                let mut tuple = serializer.serialize_tuple(2)?;
                tuple.serialize_element(&name)?;
                tuple.serialize_element(&id)?;
                tuple.end()
            }
            Self::Argument(value) => {
                let mut tuple = serializer.serialize_tuple(2)?;
                tuple.serialize_element(&value)?;
                tuple.serialize_element(&serde_json::Value::Null)?;
                tuple.end()
            }
        }
    }
}

impl<'de> serde::Deserialize<'de> for BlockField {
    fn deserialize<D>(_deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        todo!("Deserializer for BlockField is not implemented")
    }
}

#[derive(Debug, Clone)]
pub enum Value {
    Pointer(String),
    Text(String),
    Number(f64),
    Variable(String, String),
}

impl Value {
    pub fn should_shadow(&self) -> bool {
        match self {
            Self::Pointer(..) | Self::Variable(..) => false,
            _ => true,
        }
    }
}

impl serde::Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Pointer(id) => serializer.serialize_str(&id),
            Self::Text(value) => {
                let mut tuple = serializer.serialize_tuple(2)?;
                tuple.serialize_element(&10)?;
                tuple.serialize_element(value)?;
                tuple.end()
            }
            Self::Number(value) => {
                let mut tuple = serializer.serialize_tuple(2)?;
                tuple.serialize_element(&4)?;
                tuple.serialize_element(value)?;
                tuple.end()
            }
            Self::Variable(id, name) => {
                let mut tuple = serializer.serialize_tuple(3)?;
                tuple.serialize_element(&12)?;
                tuple.serialize_element(name)?;
                tuple.serialize_element(id)?;
                tuple.end()
            }
        }
    }
}

impl<'de> serde::Deserialize<'de> for Value {
    fn deserialize<D>(_deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        todo!("Deserializer for Value is not implemented")
    }
}

/// Mutations are weird things in scratch.
/// They are used for example by custom blocks to change their
/// appearance.
#[derive(Debug, Clone, Default)]
pub struct BlockMutation {
    pub proccode: String,
    pub argument_ids: Vec<String>,
    pub argument_names: Vec<String>,
    pub argument_defaults: Vec<String>,
    pub warp: bool,
}

impl serde::Serialize for BlockMutation {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut ser = serializer.serialize_struct("mutation", 7)?;
        ser.serialize_field("tagName", "mutation")?;
        ser.serialize_field::<[u8]>("children", &[])?;

        ser.serialize_field("proccode", &self.proccode)?;
        ser.serialize_field(
            "argumentids",
            &serde_json::ser::to_string(&self.argument_ids).unwrap(),
        )?;
        ser.serialize_field(
            "argumentnames",
            &serde_json::ser::to_string(&self.argument_names).unwrap(),
        )?;
        ser.serialize_field(
            "argumentdefaults",
            &serde_json::ser::to_string(&self.argument_defaults).unwrap(),
        )?;
        ser.serialize_field("warp", if self.warp { "true" } else { "false" })?;

        ser.end()
    }
}

impl<'de> serde::Deserialize<'de> for BlockMutation {
    fn deserialize<D>(_deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        todo!("Deserialize is not implemented yet for BlockMutation")
    }
}

impl serde::Serialize for ProjectBlocks {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.blocks.serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for ProjectBlocks {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(ProjectBlocks {
            blocks: HashMap::deserialize(deserializer)?,
        })
    }
}

impl Into<Value> for f64 {
    fn into(self) -> Value {
        Value::Number(self)
    }
}
impl Into<Value> for String {
    fn into(self) -> Value {
        Value::Text(self)
    }
}
