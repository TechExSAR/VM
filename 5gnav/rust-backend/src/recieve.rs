use crate::logger::Logger;
use crate::DroneCommand;
use amiquip::{
    Connection, ConsumerMessage, ConsumerOptions, ExchangeDeclareOptions, ExchangeType, FieldTable,
    QueueDeclareOptions,
};
use std::sync::mpsc::SyncSender;
use std::{thread, time};

pub fn recv_function(command_obj: SyncSender<(DroneCommand, String)>, ip_address: &str, id: &str) {
    let logging_context = format!("FROM SERVER {} ({})", id, ip_address);
    let mut recv_logger = Logger::new(logging_context.clone());

    loop {
        // Open connection.
        let mut connection = match Connection::insecure_open(&format!(
            "amqp://tmobile:tmobile123@{}/drone",
            ip_address
        )) {
            Ok(connected) => {
                recv_logger.print("connected", false);
                connected
            }
            Err(not_connected) => {
                recv_logger.print(format!("{:?}", not_connected), false);
                thread::sleep(time::Duration::from_millis(100));
                continue;
            }
        };

        // Open a channel - None says let the library choose the channel ID.
        let channel = match connection.open_channel(None) {
            Ok(channel_opened) => channel_opened,
            Err(not_opened) => {
                recv_logger.print(format!("{:?}", not_opened), false);
                continue;
            }
        };

        match channel.exchange_declare(
            ExchangeType::Fanout,
            "droneExchange",
            ExchangeDeclareOptions {
                arguments: FieldTable::default(),
                auto_delete: false,
                durable: false,
                internal: false,
            },
        ) {
            Ok(_) => {
                recv_logger.print("Exchange Declared", false);
            }
            Err(exchnage_not_declared) => {
                recv_logger.print(format!("{:?}", exchnage_not_declared), false);
            }
        }

        let queue = match channel.queue_declare("command", QueueDeclareOptions::default()) {
            Ok(queue_declared) => queue_declared,
            Err(queue_not_declared) => {
                recv_logger.print(format!("{:?}", queue_not_declared), false);
                continue;
            }
        };

        match channel.queue_bind(queue.name(), "droneExchange", "", FieldTable::default()) {
            Ok(_) => {
                //recv_logger.print("Exchange bounded to queue");
            }
            Err(exchange_queue_bind_failed) => {
                recv_logger.print(format!("{:?}", exchange_queue_bind_failed), false)
            }
        }

        // Start a consumer.
        let consumer = match queue.consume(ConsumerOptions::default()) {
            Ok(queue_consumed) => queue_consumed,
            Err(queue_not_consumed) => {
                recv_logger.print(format!("{:?}", queue_not_consumed), false);
                continue;
            }
        };
        recv_logger.print("Listening...", true);

        for message in consumer.receiver().iter() {
            match message {
                ConsumerMessage::Delivery(delivery) => {
                    let body = String::from_utf8_lossy(&delivery.body);

                    if let Ok(ok_command_obj_dsrl) = DroneCommand::dsrl(body.to_string()) {
                        recv_logger.print(format!("{:?}", ok_command_obj_dsrl), false);
                        if let Ok(_) =
                            command_obj.send((ok_command_obj_dsrl, logging_context.clone()))
                        {
                            // do nothing, command successfully sent to channel
                        } else {
                            recv_logger
                                .print("Couldn't send deserialized message to channel", false);
                        }
                    } else {
                        recv_logger.print("Couldn't deserialize bytes into json object", false);
                    }

                    if let Ok(_) = consumer.ack(delivery) {
                    } else {
                        recv_logger.print("Ack failed, potentially not delievered", false)
                    };
                }
                other => {
                    recv_logger.print(format!("Consumer ended: {:?}", other), false);
                    break;
                }
            }
        }
        if let Ok(_) = connection.close() {
        } else {
            recv_logger.print("Connection couldn't close", false)
        };
        recv_logger.print("Connection ended, restarting connection....", false);
        thread::sleep(time::Duration::from_millis(100));
    }
}
