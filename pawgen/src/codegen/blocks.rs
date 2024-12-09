use std::{cell::RefMut, collections::HashMap};

use crate::schema::{self, ProjectBlocks};

use super::generate_next_id;

pub struct BlocksBuilder<'blocks> {
    blocks: RefMut<'blocks, ProjectBlocks>,

    previous: Option<String>,
    stack: Vec<String>,
    awaiting_push: bool,

    procedures: HashMap<String, ProcedureDefinition>,
}

impl<'blocks> BlocksBuilder<'blocks> {
    pub(super) fn new(blocks: RefMut<'blocks, ProjectBlocks>) -> Self {
        Self {
            blocks,
            previous: None,
            stack: Vec::new(),
            awaiting_push: true,
            procedures: HashMap::new(),
        }
    }

    pub fn get_block_builder<'a>(&'a mut self, block_id: String) -> BlockBuilder<'a, 'blocks> {
        BlockBuilder::new(self, block_id)
    }

    fn await_push(&mut self) {
        self.awaiting_push = true
    }

    fn peek_stack(&mut self) -> String {
        self.stack.last().unwrap().clone()
    }

    fn pop_stack(&mut self) -> String {
        self.stack.pop().unwrap()
    }

    pub fn end_stack(&mut self) {
        self.stack.clear();
        self.previous = None;
        self.awaiting_push = true;
    }

    pub fn block<'a>(
        &'a mut self,
        opcode: impl AsRef<str>,
        is_expression: bool,
    ) -> BlockBuilder<'a, 'blocks> {
        let id = generate_next_id();
        self.blocks.blocks.insert(
            id.clone(),
            schema::Block {
                opcode: opcode.as_ref().to_owned(),
                parent: self.previous.clone(),
                top_level: self.previous.is_none(),
                x: if self.previous.is_none() {
                    Some(0)
                } else {
                    None
                },
                y: if self.previous.is_none() {
                    Some(0)
                } else {
                    None
                },
                ..Default::default()
            },
        );

        if self.awaiting_push {
            self.stack.push(id.clone());
            self.awaiting_push = false;
        }

        if !is_expression {
            if let Some(previous) = &self.previous {
                let previous = self.blocks.blocks.get_mut(previous).unwrap();
                previous.next = Some(id.clone());
            }

            self.previous = Some(id.clone());
        }

        BlockBuilder::new(self, id)
    }

    pub fn control_if(
        &mut self,
        condition: impl FnOnce(&mut BlocksBuilder) -> String,
        flow: impl FnOnce(&mut BlocksBuilder),
    ) {
        let control_block = self.block("control_if", false).finish(); // We need to drop mutable reference.
        let condition_id = condition(self);
        self.await_push(); // Record first block being pushed.
        flow(self);
        let substack_id = self.pop_stack();

        let mut control_block = self.get_block_builder(control_block);
        control_block
            .set_input("CONDITION", &[schema::Value::Pointer(condition_id)])
            .set_input("SUBSTACK", &[schema::Value::Pointer(substack_id)]);
    }

    pub fn control_if_else(
        &mut self,
        condition: impl FnOnce(&mut BlocksBuilder) -> String,
        flow_true: impl FnOnce(&mut BlocksBuilder),
        flow_false: impl FnOnce(&mut BlocksBuilder),
    ) {
        let control_block = self.block("control_if_else", false).finish(); // We need to drop mutable reference.
        let condition_id = condition(self);
        self.await_push(); // Record first block being pushed.
        flow_true(self);
        let substack_true_id = self.pop_stack();
        self.await_push(); // Record first block being pushed.
        flow_false(self);
        let substack_false_id = self.pop_stack();

        let mut control_block = self.get_block_builder(control_block);
        control_block
            .set_input("CONDITION", &[schema::Value::Pointer(condition_id)])
            .set_input("SUBSTACK", &[schema::Value::Pointer(substack_true_id)])
            .set_input("SUBSTACK2", &[schema::Value::Pointer(substack_false_id)]);
    }

    pub fn define_procedure(
        &mut self,
        name: impl AsRef<str>,
        arguments: HashMap<String, ProcedureArgumentType>,
        warp: bool,
    ) {
        let mut def = ProcedureDefinition::default();
        def.warp = warp;

        let proc_definition = self.block("procedures_definition", false).finish();
        let mut proc_prototype = self.block("procedures_prototype", true);
        proc_prototype.override_parent(Some(proc_definition.clone()));
        let proc_prototype = proc_prototype.finish();

        {
            let mut mutation = schema::BlockMutation::default();
            mutation.warp = warp;

            // Build proccode from arguments
            let proccode = format!(
                "{} {}",
                name.as_ref(),
                arguments
                    .iter()
                    .map(|(_, ty)| match ty {
                        ProcedureArgumentType::NumberOrText => "%s",
                        ProcedureArgumentType::Boolean => "%b",
                    })
                    .collect::<Vec<_>>()
                    .join(" ")
            );
            def.proccode = proccode.clone();
            mutation.proccode = proccode;

            let mut proto_reporters = Vec::new();
            for (name, ty) in arguments.iter() {
                let id = generate_next_id();

                // Assign arguments.
                def.arguments.push((id.clone(), name.clone()));
                mutation.argument_ids.push(id.clone());
                mutation.argument_names.push(name.clone());
                mutation.argument_defaults.push(
                    match ty {
                        ProcedureArgumentType::NumberOrText => "",
                        ProcedureArgumentType::Boolean => "false",
                    }
                    .to_string(),
                );

                // Create reporter blocks.
                let mut reporter = self.block(
                    match ty {
                        ProcedureArgumentType::NumberOrText => "argument_reporter_string_number",
                        ProcedureArgumentType::Boolean => "argument_reporter_boolean",
                    },
                    true,
                );
                reporter.override_parent(Some(proc_prototype.clone()));
                reporter.set_field("VALUE", schema::BlockField::Argument(name.clone()));

                proto_reporters.push(reporter.finish());
            }

            let mut proc_prototype = self.get_block_builder(proc_prototype);
            proc_prototype.block_ref().mutation = Some(mutation);
            for (id, reporter) in def.arguments.iter().zip(proto_reporters.iter()) {
                proc_prototype.set_input(id.0.clone(), &[schema::Value::Pointer(reporter.clone())]);
            }

            let proc_prototype = proc_prototype.finish();
            self.get_block_builder(proc_definition)
                .set_input("custom_block", &[schema::Value::Pointer(proc_prototype)]);
        }
    }

    pub fn get_arguments_for_procedure(&self, name: impl AsRef<str>) -> &Vec<(String, String)> {
        &self.procedures.get(name.as_ref()).unwrap().arguments
    }

    pub fn call_procedure(&mut self, name: impl AsRef<str>, arguments: &[schema::Value]) {
        let procedure = self.procedures.get(name.as_ref()).unwrap().clone();

        let mut call = self.block("procedures_call", false);
        call.block_ref().mutation = Some(schema::BlockMutation {
            warp: procedure.warp,
            proccode: procedure.proccode.clone(),
            argument_ids: procedure
                .arguments
                .iter()
                .map(|arg| arg.0.clone())
                .collect(),
            ..Default::default()
        });
        for (value, id) in arguments.iter().zip(procedure.arguments.iter()) {
            call.set_input(&id.0, &[value.clone()]);
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProcedureArgumentType {
    NumberOrText,
    Boolean,
}

#[derive(Clone, Default)]
pub struct ProcedureDefinition {
    pub arguments: Vec<(String, String)>,
    pub proccode: String,
    pub warp: bool,
}

pub struct BlockBuilder<'a, 'b: 'a> {
    builder: &'a mut BlocksBuilder<'b>,
    id: String,
}

impl<'a, 'b: 'a> BlockBuilder<'a, 'b> {
    fn new(builder: &'a mut BlocksBuilder<'b>, block_id: String) -> Self {
        Self {
            builder,
            id: block_id,
        }
    }

    fn block_ref(&mut self) -> &mut schema::Block {
        self.builder.blocks.blocks.get_mut(&self.id).unwrap()
    }

    pub fn override_parent(&mut self, parent: Option<String>) {
        self.block_ref().parent = parent
    }

    pub fn set_input(&mut self, name: impl AsRef<str>, values: &[schema::Value]) -> &mut Self {
        self.block_ref().inputs.insert(
            name.as_ref().to_owned(),
            schema::BlockInput {
                kind: if values[0].should_shadow() {
                    1
                } else {
                    if values.len() > 1 {
                        3
                    } else {
                        2
                    }
                },
                values: values.to_vec(),
            },
        );

        for value in values.iter() {
            if let schema::Value::Pointer(ptr) = value {
                // Override parent so we don't need to do this manually for expressions.
                self.builder.blocks.blocks.get_mut(ptr).unwrap().parent = Some(self.id());
            }
        }

        self
    }

    pub fn set_field(&mut self, name: impl AsRef<str>, field: schema::BlockField) -> &mut Self {
        self.block_ref()
            .fields
            .insert(name.as_ref().to_owned(), field);
        self
    }

    pub fn id(&self) -> String {
        self.id.clone()
    }
    pub fn finish(self) -> String {
        self.id
    }
}
