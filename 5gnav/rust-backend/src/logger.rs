use reqwest;
use std::collections::BTreeMap;
use std::sync::mpsc::{channel, Sender};
use std::{thread, time};

#[derive(Debug, Clone)]
pub struct Logger {
    context: String,
    discord_channel: Sender<String>,
}

impl Logger {
    pub fn new(context: String) -> Logger {
        // make a channel for sending log msgs to Discord
        let (sendr, recvr) = channel::<String>();

        // recvr is used here to get msgs to send to Discord
        thread::spawn(move || {
            let https_client = reqwest::blocking::Client::new();

            // this is for sending JSON-like structure to the Discord API
            let mut bmap = BTreeMap::<String, String>::new();

            for msg in recvr {
                // insert the msg to send in our map (JSON-like structure) for sending to Discord API
                bmap.insert("content".to_string(), msg.into());

                // throttle sending to avoid trigger Discord's rate-limiting (too much)
                thread::sleep(time::Duration::from_millis(1000));

                // const make this compile-time so no run-time cost
                const DISCORD_WEBHOOK_URL: &str = "https://discord.com/api/webhooks/796459823054454786/078u5beQtYN70i9PBypNjDBXGSo_EZ2Ggxeo6QuBf-9IjkuEVnWY-tAwCcJ4DLiSRJH2";

                loop {
                    // send our map (JSON-like structure) to Discord
                    match https_client.post(DISCORD_WEBHOOK_URL).json(&bmap).send() {
                        // if no errors
                        Ok(res) => {
                            // check the status code

                            // 200-299 status code means it was successful
                            if res.status().as_u16() >= 200 && res.status().as_u16() <= 299 {
                                break;
                            }

                            // 429 status code means we triggered Discord's rate-limiting
                            if res.status() == 429 {
                                // Discord includes a field in its header: "retry-after"...
                                print!("{:?}", res.headers().get("retry-after"));

                                if let Some(time_wait) = res.headers().get("retry-after") {
                                    // ...so lets "retry-after" the specified number of seconds
                                    thread::sleep(time::Duration::from_millis(
                                        if let Ok(parsed_string) = time_wait.to_str() {
                                            parsed_string.parse::<u64>().unwrap_or(2000)
                                        } else {
                                            // if, for some reason, "retry-after" doesn't exist
                                            // waiting 2 seconds its a good default
                                            2000
                                        },
                                    ))
                                } else {
                                    // if, for some reason, "retry-after" doesn't exist
                                    // waiting 2 seconds its a good default
                                    thread::sleep(time::Duration::from_millis(2000))
                                }
                            }
                            // println!("{:?}\n\n",res.status()); // Debugging to test status results from discord.
                        }

                        // if yes errors
                        Err(err) => {
                            println!("{:?}", err);
                        }
                    }
                }
            }
        });

        // sendr is used here for other code to send msgs to Discord
        Logger {
            context: context,
            discord_channel: sendr,
        }
    }

    pub fn print<S: Into<String>>(&mut self, msg: S, send_to_discord: bool) {
        let msg = format!("[{}]\t {}", self.context, msg.into());
        println!("{}", msg);
        if send_to_discord {
            self.ping_discord(msg);
        }
    }

    pub fn ping_discord<S: Into<String>>(&mut self, msg: S) {
        if let Ok(_) = self.discord_channel.send(msg.into()) {
            // do nothing, log successfully sent to discord_channel which then sends it to discord
        } else {
            // the recv channel has probably closed
            self.print(
                "Couldn't send msg over discord_channel (where it then goes to Discord)",
                false,
            )
        }
    }
}
