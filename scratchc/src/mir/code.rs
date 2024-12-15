use std::{
    cell::{Ref, RefCell},
    collections::HashMap,
    sync::Arc,
};

#[derive(Debug)]
pub struct BlockDefinitions {
    defs: RefCell<HashMap<String, BlockDefinition>>,
}

impl BlockDefinitions {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            defs: RefCell::default(),
        })
    }

    pub fn define(&self, id: impl AsRef<str>, block: BlockDefinition) {
        self.defs.borrow_mut().insert(id.as_ref().to_owned(), block);
    }

    pub fn get<'a>(&'a self, id: impl AsRef<str>) -> Ref<'a, BlockDefinition> {
        Ref::map(self.defs.borrow(), |defs| defs.get(id.as_ref()).unwrap())
    }
}

#[derive(Debug, Clone)]
pub struct BlockDefinition {
    pub(super) opcode: String,
    pub(super) is_expression: bool,
    pub(super) inputs: Vec<BlockInput>,
    pub(super) fields: Vec<BlockField>,
}

impl BlockDefinition {
    pub fn new(
        opcode: impl AsRef<str>,
        is_expression: bool,
        inputs: impl IntoIterator<Item = BlockInput>,
        fields: impl IntoIterator<Item = BlockField>,
    ) -> Self {
        Self {
            opcode: opcode.as_ref().to_owned(),
            is_expression,
            inputs: inputs.into_iter().collect(),
            fields: fields.into_iter().collect(),
        }
    }
}

#[derive(Debug, Clone, derive_more::Constructor)]
pub struct BlockInput {
    pub(super) name: String,
    pub(super) ty: DataType,
}

#[derive(Debug, Clone, derive_more::Constructor)]
pub struct BlockField {
    pub(super) name: String,
    pub(super) value: String,
}

#[derive(Debug, Clone, PartialEq, derive_more::Unwrap)]
pub enum DataType {
    // Primitive types
    Text,
    Number,
    Boolean,

    // Advanced types
    Structure(Vec<DataType>),
}

impl DataType {
    pub fn is_primitive(&self) -> bool {
        matches!(self, Self::Text | Self::Boolean | Self::Number)
    }

    pub fn calculate_size(&self) -> usize {
        self.flatten().len()
    }

    pub fn flatten(&self) -> Vec<DataType> {
        match self {
            DataType::Structure(fields) => fields
                .iter()
                .map(|field| field.flatten())
                .flatten()
                .collect(),
            x @ _ => vec![x.clone()],
        }
    }
}

#[derive(Debug, Clone)]
pub struct Procedure {
    pub(super) name: String,
    pub(super) is_warp: bool,
    pub(super) inputs: Vec<DataType>,
    pub(super) block: CodeBlock,
}

impl Procedure {
    pub fn new(
        name: impl AsRef<str>,
        is_warp: bool,
        inputs: impl IntoIterator<Item = DataType>,
    ) -> Self {
        Self {
            name: name.as_ref().to_owned(),
            is_warp,
            inputs: inputs.into_iter().collect(),
            block: CodeBlock::default(),
        }
    }

    pub fn code_block(&mut self) -> &mut CodeBlock {
        &mut self.block
    }
}

#[derive(Debug, Clone, Default)]
pub struct CodeBlock {
    pub(super) code: Vec<Statement>,
}

impl CodeBlock {
    pub fn push_stmt(&mut self, stmt: Statement) -> &mut Self {
        self.code.push(stmt);
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Constant(pawgen::schema::Value),
    BlockCall(String, Vec<Statement>),
    ArgumentRef(usize, DataType),
    VariableRef(String, String, DataType),
    Assignment(Box<Statement>, Box<Statement>),
    FieldRef(Box<Statement>, usize, DataType),
    StructureLiteral(Vec<pawgen::schema::Value>, DataType),
}
