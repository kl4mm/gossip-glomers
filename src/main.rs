use gossip_glomers::maelstrom::{Message, Node, NodeError, Type, NODE_ID};
use serde_json::json;

fn main() {
    let mut node = Node::new();

    node.handle(Type::Init, init);
    node.handle(Type::Echo, echo);
    node.handle(Type::Generate, generate);
    node.handle(Type::Broadcast, broadcast);
    node.handle(Type::Read, read);
    node.handle(Type::Topology, topology);

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
    use std::time::{SystemTime, UNIX_EPOCH};

    let mut uid = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    uid &= 0x0000000000000000FFFFFFFFFFFFFFFF;
    uid <<= 16;
    uid |= rand::random::<u64>() as u128;

    msg.body.message_type = Type::GenerateOk;
    msg.body.fields.insert("id".into(), json!(uid));

    Node::reply(msg).map_err(|_| NodeError::Abort)?;

    Ok(())
}

static mut MESSAGES: Vec<u64> = Vec::new();

pub fn broadcast(mut msg: Message) -> Result<(), NodeError> {
    let message = msg.body.fields["message"].as_u64().unwrap();

    // SAFETY: Node is single threaded
    unsafe {
        MESSAGES.push(message);
    }

    msg.body.message_type = Type::BroadcastOk;
    msg.body.fields.remove("message");

    Node::reply(msg).map_err(|_| NodeError::Abort)?;

    Ok(())
}

pub fn read(mut msg: Message) -> Result<(), NodeError> {
    msg.body.message_type = Type::ReadOk;

    // SAFETY: Node is single threaded
    unsafe {
        msg.body.fields.insert("messages".into(), json!(MESSAGES));
    }

    Node::reply(msg).map_err(|_| NodeError::Abort)?;

    Ok(())
}

pub fn topology(mut msg: Message) -> Result<(), NodeError> {
    let t = msg
        .body
        .fields
        .get("topology")
        .unwrap()
        .as_object()
        .unwrap();
    // .to_owned();

    msg.body.fields.remove("topology");

    msg.body.message_type = Type::TopologyOk;

    Node::reply(msg).map_err(|_| NodeError::Abort)?;

    Ok(())
}
