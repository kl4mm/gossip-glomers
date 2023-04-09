use gossip_glomers::maelstrom::{Message, Node, NodeError, Type, NODE_ID};

fn main() {
    let mut node = Node::new();

    node.handle(Type::Init, init);
    node.handle(Type::Echo, echo);
    node.handle(Type::Generate, generate);

    if let Err(e) = node.run() {
        eprintln!("ERROR: {e}");
        std::process::exit(1);
    };
}

pub fn init(mut msg: Message) -> Result<(), NodeError> {
    msg.body.message_type = Type::InitOk;

    let node_id = msg.body.fields["node_id"].as_str().unwrap().to_owned();
    NODE_ID.set(node_id).unwrap();

    Node::reply(msg).map_err(|_| NodeError::Abort)?;

    Ok(())
}

pub fn echo(mut msg: Message) -> Result<(), NodeError> {
    msg.body.message_type = Type::EchoOk;

    Node::reply(msg).map_err(|_| NodeError::Abort)?;

    Ok(())
}

pub fn generate(mut msg: Message) -> Result<(), NodeError> {
    use serde_json::json;
    use std::time::{SystemTime, UNIX_EPOCH};

    let mut uid = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();

    uid &= 0x00000000FFFFFFFF;
    uid <<= 8;
    uid |= rand::random::<u64>() as u128;

    msg.body.message_type = Type::GenerateOk;
    msg.body.fields.insert("id".into(), json!(uid));

    Node::reply(msg).map_err(|_| NodeError::Abort)?;

    Ok(())
}
