use pawgen::{codegen::ProjectBuilder, schema::Value};

fn main() {
    // Currently this is only temporary for testing purposes.
    let mut project = ProjectBuilder::new();
    project.init_core();

    {
        let background = project
            .register_asset("background1", "../prototyping/background.svg")
            .unwrap();
        let stage = project.get_stage();
        stage.add_costume(&background);
        stage.set_default_costume("background1");
    }

    {
        let cat_sprite = project
            .register_asset("cat1", "../prototyping/cat.svg")
            .unwrap();
        let cat = project.create_sprite("cat");
        cat.add_costume(&cat_sprite);
        cat.set_default_costume("cat1");

        let mut code = cat.blocks_builder();
        code.block("event_whenflagclicked", false);
        code.block("motion_movesteps", false)
            .set_input("STEPS", &[Value::Number(10.0)]);

        code.control_if(
            |code| {
                code.block("operator_equals", true)
                    .set_input("OPERAND1", &[Value::Text("A".to_string())])
                    .set_input("OPERAND2", &[Value::Text("A".to_string())])
                    .id()
            },
            |code| {
                code.block("looks_say", false)
                    .set_input("MESSAGE", &[Value::Text("A is equal to A!".to_string())]);
            },
        );
    }

    project
        .bundle_project("../prototyping/test_project.sb3")
        .unwrap();
}
