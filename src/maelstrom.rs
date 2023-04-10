use std::{
    collections::HashMap,
    io::{self, Write},
};

use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub static NODE_ID: OnceCell<String> = OnceCell::new();

#[derive(Deserialize, Serialize)]
pub struct Message {
    src: String,
    dest: String,
    pub body: Body,
}

#[derive(Deserialize, Serialize)]
pub struct Body {
    #[serde(rename = "type")]
    pub message_type: Type,
    msg_id: Option<u32>,
    in_reply_to: Option<u32>,

    #[serde(flatten)]
    pub fields: HashMap<String, Value>,
}

#[derive(Deserialize, Serialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum Type {
    Init,
    InitOk,
    Error,
    Echo,
    EchoOk,
    Generate,
    GenerateOk,
    Broadcast,
    BroadcastOk,
    Read,
    ReadOk,
    Topology,
    TopologyOk,
}

type Handler = fn(Message) -> Result<(), NodeError>;
pub struct Node {
    handlers: HashMap<Type, Handler>,
}

pub enum NodeError {
    Abort,
    TemporarilyUnavailable,
}

impl Node {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::default(),
        }
    }

    pub fn handle(&mut self, message_type: Type, handler: Handler) {
        self.handlers.insert(message_type, handler);
    }

    pub fn reply(mut msg: Message) -> io::Result<()> {
        let mut stdout = io::stdout();

        msg.dest = msg.src;
        msg.body.in_reply_to = msg.body.msg_id;
        msg.src = match NODE_ID.get() {
            Some(m) => m.to_owned(),
            None => {
                unimplemented!()
            }
        };

        let mut msg = serde_json::to_string(&msg).unwrap();
        msg.push('\n');

        stdout.write_all(msg.as_bytes())?;
        stdout.flush()?;

        Ok(())
    }

    pub fn run(&mut self) -> io::Result<()> {
        let stdin = io::stdin();
        let mut msgs = serde_json::Deserializer::from_reader(stdin).into_iter::<Message>();

        while let Some(msg) = msgs.next() {
            let msg = match msg {
                Ok(m) => m,
                Err(e) => {
                    panic!("{e}");
                }
            };

            // TODO: error reply
            let handle = match self.handlers.get(msg.message_type()) {
                Some(h) => h,
                None => {
                    unimplemented!("handler doesn't exist")
                }
            };

            if let Err(_e) = handle(msg) {
                unimplemented!("handle failed")
            };
        }

        Ok(())
    }
}

impl Message {
    pub fn message_type(&self) -> &Type {
        &self.body.message_type
    }

    pub fn is_init(&self) -> bool {
        self.body.message_type.is_init()
    }
}

impl Type {
    pub fn is_init(&self) -> bool {
        *self == Type::Init
    }
}
