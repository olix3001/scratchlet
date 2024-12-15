use std::{collections::HashMap, sync::Arc};

use pawgen::codegen;

use super::{BlockDefinitions, CodeBlock, DataType, Procedure, Project, Sprite, Statement};

/// Refinery that converts MIR into pawgen Project.
pub struct MirRefinery {
    config: MirRefinementConfig,
    block_definitions: Arc<BlockDefinitions>,
}

#[derive(Debug)]
pub struct MirRefinementConfig {
    /// Whether to use thread variables from Turbowarp's "Temporary Variables" extension.
    use_thread_variables: bool,
}

impl Default for MirRefinementConfig {
    fn default() -> Self {
        Self {
            use_thread_variables: false,
        }
    }
}

impl MirRefinery {
    pub fn new(config: MirRefinementConfig) -> Self {
        Self {
            config,
            block_definitions: BlockDefinitions::new(),
        }
    }

    pub fn refine_project(&mut self, project: Project) -> codegen::ProjectBuilder {
        self.block_definitions = project.block_definitions.clone();
        let mut builder = codegen::ProjectBuilder::new();

        for sprite in project.sprites.iter() {
            self.refine_sprite(sprite, &mut builder);
        }

        builder
    }

    fn refine_sprite(
        &self,
        sprite: &Sprite,
        builder: &mut codegen::ProjectBuilder,
    ) -> codegen::SpriteBuilder {
        let sb = builder.create_sprite(&sprite.name);
        if sprite.is_stage {
            sb.set_stage(true);
        }

        for costume in sprite.costumes.iter() {
            let asset = builder
                .register_asset(&costume.name, &costume.source)
                .expect("Costume file should exist at this point");
            sb.add_costume(&asset);
        }
        if let Some(first) = sprite.costumes.first() {
            sb.set_default_costume(&first.name);
        }

        for sound in sprite.sounds.iter() {
            let _asset = builder
                .register_asset(&sound.name, &sound.source)
                .expect("Sound file should exist at this point");
            // TODO: sb.add_sound(&asset);
        }

        for procedure in sprite.procedures.iter() {
            self.refine_procedure(procedure, &sb);
        }

        sb
    }

    fn refine_procedure(&self, procedure: &Procedure, sb: &codegen::SpriteBuilder) {
        let mut arguments = HashMap::new();

        for (ii, input) in procedure.inputs.iter().enumerate() {
            for (fi, field) in input.flatten().iter().enumerate() {
                let arg_name = format!("__arg_{ii}:{fi}");
                arguments.insert(arg_name, self.refine_datatype_into_procargtype(field));
            }
        }

        let mut bb = sb.blocks_builder();
        bb.define_procedure(&procedure.name, arguments, procedure.is_warp);

        self.refine_codeblock(&procedure.block, &mut bb);

        bb.end_stack();
    }

    fn refine_codeblock(&self, codeblock: &CodeBlock, bb: &mut codegen::BlocksBuilder) {
        for stmt in codeblock.code.iter() {
            self.refine_stmt(stmt, bb);
        }
    }

    fn refine_stmt(&self, stmt: &Statement, bb: &mut codegen::BlocksBuilder) -> Option<DataValue> {
        match stmt {
            Statement::Constant(value) => Some(DataValue::Primitive(value.clone())),
            Statement::BlockCall(id, args) => {
                let def = self.block_definitions.get(id);
                let call_values: Vec<_> = args
                    .iter()
                    .map(|arg| {
                        self.refine_stmt(arg, bb)
                            .expect("Arguments should be values")
                            .into_primitive(bb)
                    })
                    .collect();
                let mut b = bb.block(&def.opcode, def.is_expression);

                for (input, value) in def.inputs.iter().zip(call_values) {
                    b.set_input(&input.name, &[value]);
                }

                for field in def.fields.iter() {
                    b.set_field(
                        &field.name,
                        pawgen::schema::BlockField::Argument(field.value.clone()),
                    );
                }

                Some(DataValue::Primitive(pawgen::schema::Value::Pointer(
                    b.finish(),
                )))
            }
            Statement::ArgumentRef(index, dt) => Some(DataValue::Argument(*index, dt.clone())),
            Statement::VariableRef(id, name, dt) => {
                Some(DataValue::Variable(id.clone(), name.clone(), dt.clone()))
            }
            Statement::StructureLiteral(values, dt) => {
                Some(DataValue::StructureLiteral(values.clone(), dt.clone()))
            }
            Statement::FieldRef(target, index, dt) => Some(DataValue::Field(
                Box::new(self.refine_stmt(target, bb).unwrap()),
                *index,
                dt.clone(),
            )),
            Statement::Assignment(target, value) => {
                let (id, name, dt) = match self.refine_stmt(&*target, bb).unwrap() {
                    DataValue::Variable(id, name, dt) => (id, name, dt),
                    ref v @ DataValue::Field(_, index, ref sdt) => {
                        let pawgen::schema::Value::Variable(id, name) =
                            v.clone().into_primitive(bb)
                        else {
                            unreachable!()
                        };
                        (id, name, sdt.clone().unwrap_structure()[index].clone())
                    }
                    _ => unreachable!(),
                };

                if dt.is_primitive() {
                    self.write_primitive_variable(
                        id,
                        name,
                        self.refine_stmt(value, bb).unwrap(),
                        bb,
                    );
                } else {
                    let rv = self.refine_stmt(value, bb).unwrap();
                    let rvdt = rv.get_data_type();

                    let is_suffixed = id.contains(":");
                    let id_prefix = if is_suffixed {
                        id.split(":").next().unwrap()
                    } else {
                        &id
                    };
                    let name_prefix = if is_suffixed {
                        id.split(":").next().unwrap()
                    } else {
                        &name
                    };
                    let start_index: usize = if is_suffixed {
                        id.split(":").last().unwrap().parse().unwrap()
                    } else {
                        0
                    };

                    for (i, _) in dt.flatten().iter().enumerate() {
                        self.write_primitive_variable(
                            format!("{id_prefix}:{}", start_index + i),
                            format!("{name_prefix}:{}", start_index + i),
                            DataValue::Field(Box::new(rv.clone()), i, rvdt.clone()),
                            bb,
                        );
                    }
                }

                None
            }
        }
    }

    fn write_primitive_variable(
        &self,
        id: String,
        name: String,
        value: DataValue,
        bb: &mut codegen::BlocksBuilder,
    ) {
        let value = value.into_primitive(bb);
        bb.block("data_setvariableto", false)
            .set_field("VARIABLE", pawgen::schema::BlockField::Variable(id, name))
            .set_input("VALUE", &[value]);
    }

    fn refine_datatype_into_procargtype(
        &self,
        datatype: &DataType,
    ) -> codegen::ProcedureArgumentType {
        match datatype {
            DataType::Text | DataType::Number => codegen::ProcedureArgumentType::NumberOrText,
            DataType::Boolean => codegen::ProcedureArgumentType::Boolean,
            _ => unreachable!("Datatype should be flattened before conversion"),
        }
    }
}

#[derive(Debug, Clone, derive_more::IsVariant, derive_more::Unwrap)]
enum DataValue {
    Primitive(pawgen::schema::Value),
    Argument(usize, DataType),
    Variable(String, String, DataType),
    Field(Box<DataValue>, usize, DataType),
    StructureLiteral(Vec<pawgen::schema::Value>, DataType),
}

impl DataValue {
    pub fn get_data_type(&self) -> DataType {
        match self {
            Self::Argument(_, dt) => dt.clone(),
            Self::Field(_, index, dt) => match dt {
                DataType::Structure(fields) => fields[*index].clone(),
                _ => unreachable!(),
            },
            Self::Variable(_, _, dt) => dt.clone(),
            Self::StructureLiteral(_, dt) => dt.clone(),
            _ => unreachable!("Getting data type of {self:?} is prohibited"),
        }
    }

    pub fn into_primitive(self, bb: &mut codegen::BlocksBuilder) -> pawgen::schema::Value {
        match self {
            Self::Primitive(value) => value,
            Self::Argument(index, dt) => pawgen::schema::Value::Pointer(
                bb.block(
                    if matches!(dt, DataType::Boolean) {
                        "argument_reporter_boolean"
                    } else {
                        "argument_reporter_string_number"
                    },
                    true,
                )
                .set_field(
                    "VALUE",
                    pawgen::schema::BlockField::Argument(format!("__arg_{index}:0")),
                )
                .id(),
            ),
            Self::Variable(id, name, _) => pawgen::schema::Value::Variable(id, name),
            f @ Self::Field(..) => {
                let (collected_fields, target) = f.collect_field_indices();
                let t = target.clone().unwrap().into_primitive(bb);
                match target.unwrap() {
                    Self::Argument(..) => {
                        let pawgen::schema::Value::Pointer(p) = t else {
                            unreachable!()
                        };
                        let mut bb = bb.get_block_builder(p.clone());
                        let br = bb.block_ref();
                        let pawgen::schema::BlockField::Argument(previous_name) =
                            br.fields.get("VALUE").unwrap()
                        else {
                            unreachable!()
                        };
                        br.fields.insert(
                            "VALUE".to_owned(),
                            pawgen::schema::BlockField::Argument(format!(
                                "{}:{}",
                                previous_name
                                    .split(":")
                                    .collect::<Vec<_>>()
                                    .first()
                                    .unwrap(),
                                collected_fields
                            )),
                        );
                        pawgen::schema::Value::Pointer(p)
                    }
                    Self::Variable(..) => {
                        let pawgen::schema::Value::Variable(id, name) = t else {
                            unreachable!()
                        };
                        pawgen::schema::Value::Variable(
                            format!("{id}:{collected_fields}"),
                            format!("{name}:{collected_fields}"),
                        )
                    }
                    Self::StructureLiteral(values, ..) => values[collected_fields].clone(),
                    _ => todo!(),
                }
            }
            Self::StructureLiteral(..) => pawgen::schema::Value::Text(
                "IF YOU SEE THIS REPORT PROBLEM WITH SCRATCHLET!!!".to_owned(),
            ),
        }
    }

    fn collect_field_indices(&self) -> (usize, Option<DataValue>) {
        match self {
            Self::Field(target, index, dt) => {
                if let DataType::Structure(fields) = dt {
                    let mut n = 0;
                    for i in 0..*index {
                        n += fields[i].calculate_size();
                    }
                    let (tn, tt) = target.collect_field_indices();
                    (n + tn, Some(tt.unwrap_or_else(|| *target.clone())))
                } else {
                    panic!("Cannot access field on non-structure/array value {self:?}")
                }
            }
            _ => (0, None),
        }
    }
}
