extern crate futures;
extern crate grpcio;
extern crate protos;
extern crate shrust;

use grpcio::{ChannelBuilder, EnvBuilder};

use std::collections::HashMap;
use std::env;
use std::sync::Arc;

use protos::simpledb::{Attribute, Attribute_Type, Entry, KeyMsg, ReadMsg};
use protos::simpledb_grpc::SimpleDbClient;

use shrust::{Shell, ShellIO};
use std::io::prelude::*;

fn new_read_msg(key: String, attributes: Vec<String>) -> ReadMsg {
    let mut msg = ReadMsg::new();
    msg.set_key(key);
    msg.set_attributes(protobuf::RepeatedField::from_vec(attributes));
    msg
}

fn new_entry_msg(key: String, values: HashMap<String, Vec<u8>>) -> Entry {
    let mut msg = Entry::new();
    msg.set_key(key);

    let mut attributes: Vec<Attribute> = Vec::new();
    for (name, value) in &values {
        let mut attribute = Attribute::new();
        attribute.set_name(name.clone());
        attribute.set_value(value.clone());
        attribute.set_field_type(Attribute_Type::BYTES);
        attributes.push(attribute);
    }
    msg.set_attributes(protobuf::RepeatedField::from_vec(attributes));
    msg
}

fn new_key_msg(key: String) -> KeyMsg {
    let mut msg = KeyMsg::new();
    msg.set_key(key);
    msg
}

fn main() {
    let args = env::args().collect::<Vec<_>>();
    if args.len() != 2 {
        panic!("Expected exactly one argument, the port to connect to.")
    }
    let port = args[1]
        .parse::<u16>()
        .unwrap_or_else(|_| panic!("{} is not a valid port number", args[1]));

    let env = Arc::new(EnvBuilder::new().build());
    let chan = ChannelBuilder::new(env).connect(format!("localhost:{}", port).as_str());
    let client = SimpleDbClient::new(chan);

    let mut shell = Shell::new(client);

    shell.new_command("read", "read entry from db", 1, |io, client, args| {
        let msg = new_read_msg(args[0].to_string(), vec![String::from("value")]);
        let reply = client.read_rpc(&msg)?;
        writeln!(io, "Reply: ${:?}", reply)?;
        Ok(())
    });

    shell.new_command("write", "write entry to db", 2, |io, client, args| {
        let key = args[0].to_string();
        let mut values: HashMap<String, Vec<u8>> = HashMap::new();

        let arr = args[1].split(",");
        for item in arr {
            let arr: Vec<&str> = item.split(":").collect();
            values.insert(String::from(arr[0]), String::from(arr[1]).into_bytes());
        }
        let msg = new_entry_msg(key, values);
        client.insert_rpc(&msg)?;
        writeln!(io, "write success")?;
        Ok(())
    });

    shell.new_command("delete", "delete entry from db", 1, |io, client, args| {
        let msg = new_key_msg(args[0].to_string());
        client.delete_rpc(&msg)?;
        writeln!(io, "delete success")?;
        Ok(())
    });

    shell.run_loop(&mut ShellIO::default());
}
