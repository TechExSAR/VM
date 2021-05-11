use lru_time_cache::LruCache;
pub mod logger;
use crate::logger::Logger;
mod utils;
use std::process::Command;
use std::sync::mpsc::sync_channel;
use std::thread;
use utils::DroneCommand;
mod recieve;
use recieve::recv_function;
use std::sync::mpsc::SyncSender;

// Server ID and IP matrix array
const ID_AND_IP: [[&'static str; 2]; 4] = [
    ["Appleberry", "54.149.182.55"],
    ["Boysenberry", "54.202.58.152"],
    ["Cranberry", "52.26.148.14"],
    ["Dewberry", "34.221.169.227"],
];

// DJI utils
const DJI_SDK_KEYS_PATH: &str = "/home/pi/new_drone/DJI-SDK/__keys__/UserCred.txt";
const CAMERA_CMD: &str = "/home/pi/new_drone/DJI-SDK/build/bin/camera";
const LAND_CMD: &str = "/home/pi/new_drone/DJI-SDK/build/bin/land";
const FLY_CMD: &str = "/home/pi/new_drone/DJI-SDK/build/bin/fly";

fn start_threads(sendr: SyncSender<(DroneCommand, String)>) {
    // Sets up 4 Rabbitmq's to listen to the frontend.
    for x in 0..4 {
        let sndr = sendr.clone();
        thread::spawn(move || recv_function(sndr, ID_AND_IP[x][1], ID_AND_IP[x][0]));
    }
}

fn call_drone_command(cmd: DroneCommand, debug: String) {
    let mut command_logger = Logger::new(format!("{} | Command: {}", debug, cmd.command.clone()));
    match cmd.command.as_str() {
        "pic" => match Command::new(CAMERA_CMD)
            .arg(DJI_SDK_KEYS_PATH)
            .arg(cmd.flightstring)
            .output()
        {
            Ok(command_ok) => {
                command_logger.print(format!("[{:?}]", command_ok), true);
            }
            Err(command_err) => {
                command_logger.print(format!("[{:?}]", command_err), true);
            }
        },
        "fly" => match Command::new(FLY_CMD)
            .arg(DJI_SDK_KEYS_PATH)
            .arg(cmd.flightstring)
            .output()
        {
            Ok(command_ok) => {
                command_logger.print(format!("[{:?}]", command_ok), true);
            }
            Err(command_err) => {
                command_logger.print(format!("[{:?}]", command_err), true);
            }
        },
        "land" => match Command::new(LAND_CMD)
            .arg(DJI_SDK_KEYS_PATH)
            .arg(cmd.flightstring.to_string())
            .output()
        {
            Ok(command_ok) => {
                command_logger.print(format!("[{:?}]", command_ok), true);
            }
            Err(command_err) => {
                command_logger.print(format!("[{:?}]", command_err), true);
            }
        },
        _ => {
            command_logger.print(
                "Command not found, update command list in main.rs to add functionality",
                true,
            );
        }
    }
}

fn main() {
    // Set up a sync channel that allows for multiple senders and one responder, that will be sent to start threads that makes 4.
    let (sendr, recvr) = sync_channel::<(DroneCommand, String)>(1);

    // Setting up a least resently used cache that deletes commands after 10 minutes of being in there.
    let mut _lru_cache =
        LruCache::<String, String>::with_expiry_duration(::std::time::Duration::from_secs(600));

    start_threads(sendr);

    for cmd_and_debug in recvr {
        let cmd = cmd_and_debug.clone().0;
        let debug = cmd_and_debug.clone().1;
        // if None is the responce, that command uuid wasn't found, call drone_command, else ignore.
        if _lru_cache.insert(cmd.uuid.clone(), cmd.command.clone()) == None {
            call_drone_command(cmd, debug);
        }
    }
}
