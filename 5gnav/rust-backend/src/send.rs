use amiquip::{Connection, Exchange, Publish};

pub fn send_function(drone_command: &String) {
    // Open connection.
    let mut connection =
        Connection::insecure_open("amqp://tmobile:tmobile123@54.149.182.55:5672/drone").unwrap();

    // Open a channel - None says let the library choose the channel ID.
    let channel = connection.open_channel(None).unwrap();

    // Get a handle to the direct exchange on our channel.
    let exchange = Exchange::direct(&channel);

    // Publish a message to the "command" queue.
    exchange
        .publish(Publish::new(drone_command.as_bytes(), "command"))
        .unwrap();

    connection.close().unwrap();
}

// channel.basic_publish(
//     "droneExchange",
//     Publish {
//         body: "".as_bytes(), // is this our message we want to send?
//         routing_key: "command".to_string(),
//         mandatory: true,
//         immediate: false,
//         properties: AmqpProperties {},
//     },
// );
