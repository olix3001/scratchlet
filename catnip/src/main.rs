use scratchc::{common::location::SourceId, frontend::parser::lexer::Tokens};

fn main() {
    const EXAMPLE: &'static str = "
    sprite Stage {
        costumes {
            @default background1: \"../prototyping/background.svg\"
        }
    }
    ";

    let mut tokens = Tokens::from_string(EXAMPLE, SourceId::dummy());
    loop {
        println!("{:?}", tokens.next().unwrap());
    }
}
