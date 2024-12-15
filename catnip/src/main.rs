use scratchc::mir::{
    self, BlockDefinition, BlockInput, Costume, DataType, MirRefinementConfig, MirRefinery,
    Procedure, Sprite, Statement,
};

fn main() {
    let mut project = mir::Project::new();
    project.get_definitions().define(
        "move",
        BlockDefinition::new(
            "motion_movesteps",
            false,
            [BlockInput::new("STEPS".to_owned(), DataType::Number)],
            [],
        ),
    );

    let mut stage = Sprite::new("Stage");
    stage
        .mark_as_stage()
        .add_costume(Costume::new("background1", "../prototyping/background.svg"));
    project.add_sprite(stage);

    let mut cat = Sprite::new("Cat");

    let mut move_right_proc = Procedure::new(
        "move_right",
        false,
        [mir::DataType::Structure(vec![
            DataType::Number,
            DataType::Boolean,
        ])],
    );
    move_right_proc
        .code_block()
        .push_stmt(Statement::Assignment(
            Box::new(Statement::VariableRef(
                "hello".to_owned(),
                "world".to_owned(),
                DataType::Structure(vec![DataType::Number, DataType::Boolean]),
            )),
            Box::new(Statement::ArgumentRef(
                0,
                DataType::Structure(vec![DataType::Number, DataType::Boolean]),
            )),
        ))
        .push_stmt(Statement::BlockCall(
            "move".to_owned(),
            vec![Statement::Constant(
                scratchc::pawgen::schema::Value::Number(10.0),
            )],
        ))
        .push_stmt(Statement::Assignment(
            Box::new(Statement::FieldRef(
                Box::new(Statement::VariableRef(
                    "lorem".to_owned(),
                    "ipsum".to_owned(),
                    DataType::Structure(vec![
                        DataType::Number,
                        DataType::Structure(vec![DataType::Boolean, DataType::Boolean]),
                    ]),
                )),
                1,
                DataType::Structure(vec![
                    DataType::Number,
                    DataType::Structure(vec![DataType::Boolean, DataType::Boolean]),
                ]),
            )),
            Box::new(Statement::StructureLiteral(
                vec![
                    scratchc::pawgen::schema::Value::Text("false".to_owned()),
                    scratchc::pawgen::schema::Value::Text("true".to_owned()),
                ],
                DataType::Structure(vec![DataType::Boolean, DataType::Boolean]),
            )),
        ));

    cat.add_costume(Costume::new("cat1", "../prototyping/cat.svg"))
        .add_procedure(move_right_proc);
    project.add_sprite(cat);

    let mut refinery = MirRefinery::new(MirRefinementConfig::default());
    let project = refinery.refine_project(project);
    project
        .bundle_project("../prototyping/test_project.sb3")
        .unwrap();
}
