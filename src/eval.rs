use crate::lexer::Span;
use crate::parser::{parse, Atom, AtomData, Command};

use std::collections::HashMap;

pub trait CommandImpl {
    const NAME: &'static str;

    fn call_impl(&self, args: &[Atom]);
}

pub trait NativeCommand {
    fn name(&self) -> &'static str;
    fn call(&self, args: &[Atom]);
}

impl<T> NativeCommand for T
where
    T: CommandImpl,
{
    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn call(&self, args: &[Atom]) {
        self.call_impl(args);
    }
}

#[derive(Clone)]
pub enum Value {
    Number(u64),
    String(String),

    Command(&'static dyn NativeCommand),
}

pub struct Echo;

impl CommandImpl for Echo {
    const NAME: &'static str = "echo";

    fn call_impl(&self, args: &[Atom]) {
        if let AtomData::String(string) = &args[0].data {
            println!("{}", string)
        }
    }
}

pub struct Env {
    env: HashMap<String, Value>,
}

impl Env {
    fn add_command(&mut self, command: &'static dyn NativeCommand) {
        self.env
            .insert(command.name().to_string(), Value::Command(command));
    }

    fn new() -> Env {
        let mut env = Env {
            env: HashMap::new(),
        };

        env.add_command(&Echo);

        env
    }

    fn get(&self, name: &str) -> Option<&Value> {
        self.env.get(&name.to_string())
    }
}

pub fn eval(source: &str) {
    let ast = parse(source).unwrap();
    let env = Env::new();

    let AtomData::Batch(body) = ast.data() else {
        todo!()
    };

    for command in body.iter() {
        let data = command.data();

        let [Atom {
            data: AtomData::Identifier(ident),
            ..
        }, ..] = &data[..]
        else {
            continue;
        };

        let Some(Value::Command(cmd)) = env.get(&ident) else {
            todo!("{ident}");
        };

        cmd.call(&data[1..]);
    }
}
