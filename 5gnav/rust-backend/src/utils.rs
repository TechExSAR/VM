use serde::{Deserialize, Serialize};
use serde_json;
// use uuid::Uuid;
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DroneCommand {
    pub uuid: String,
    pub command: String,
    pub flightstring: String,
}

impl DroneCommand {
    // Test code.
    // pub fn new(command: String) -> DroneCommand {
    //     DroneCommand {
    //         uuid: Uuid::new_v4().to_string(),
    //         command: command,
    //     }
    // }

    // pub fn srl(&self) -> serde_json::Result<String> {
    //     // To String
    //     serde_json::to_string(self)
    // }

    pub fn dsrl(string_obj: String) -> serde_json::Result<DroneCommand> {
        // To Obj
        serde_json::from_str(&string_obj)
    }
}
