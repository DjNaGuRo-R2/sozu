use env_logger;
use std::env;

use amqp::{Session,Table,Basic};

use rustc_serialize::json;

use std::thread;
use std::sync::mpsc::{Sender};
use bus::Message;
use messages::Command;

pub fn init_rabbitmq(bus_tx: Sender<Message>) {
    env_logger::init().unwrap();

    let connection_uri = &env::var("AMQP_URI").ok().expect("No RabbitMQ connection provided");
    let mut session = Session::open_url(connection_uri).ok().expect("Can't create session");
    info!("Openned session");
    let mut channel = session.open_channel(1).ok().expect("Error openning channel 1");
    debug!("Openned channel: {:?}", channel.id);

    let queue_name = "test_queue";
    let queue_declare = channel.queue_declare(queue_name, false, true, false, false, false, Table::new());

    debug!("Queue declare: {:?}", queue_declare);
    channel.basic_prefetch(10);
    debug!("Declaring get iterator...");

    thread::spawn(move || {
      loop {
        let get_results = channel.basic_get(queue_name, true);

        for m in get_results {
            let payload: Result<Command, &str> =
                String::from_utf8(m.body).map_err(|_| "Invalid payload data")
                .and_then(|s| json::decode(&s).map_err(|_| "Invalid payload structure"));

            match payload {
              Ok(command) => {
                  bus_tx.send(Message::Msg(command.clone()));
              }
              Err(e) => debug!("{}", e)
            }
        }
      }
    });
}
