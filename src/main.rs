extern crate reqwest;
extern crate serde_json;

use std::collections::VecDeque;
use std::sync::{Arc, mpsc, Mutex};
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

use reqwest::StatusCode;
use std::io::Error;

#[derive(Debug)]
struct Instance {
    uri: String,
    description: String,
    email: String,
    version: String,
    instance_type: String,
    // registrations: bool,
}

fn main() -> Result<(), reqwest::Error> {
    let mut job_queue = VecDeque::new();
    job_queue.push_back("dev.layer8.space".to_string());
    job_queue.push_back("dev.layer8.space".to_string());

    let (tx, rx) = mpsc::channel();

    let counter = Arc::new(Mutex::new(job_queue));
    let mut handles: Vec<JoinHandle<()>> = vec![];

    thread::spawn(move || {
        for received in rx {
            println!("Got: {}", received);
        }
    });

    for thread_nr in 1..5 {
        let tx_inner = mpsc::Sender::clone(&tx);
        let counter = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            let mut retry_count = 0;

            loop {
                let mut target_uri;
                {
                    let mut queue = counter.lock().unwrap();
                    target_uri = queue.pop_back();
                } // Lock is released after scope of `queue` ends

                match target_uri {
                    None => {
                        tx_inner.send(format!("T{}: Empty Queue (retry {})", thread_nr, retry_count)).unwrap();
                        retry_count = retry_count + 1;
                        thread::sleep(Duration::from_millis(120));
                    }

                    _ => {
                        let uri = target_uri.unwrap();
                        match tx_inner.send(format!("T{}: {}", thread_nr, uri)) {
                            Err(_) => eprintln!("Backfeed failed!"),
                            Ok(_) => {}
                        }
                        retry_count = 0;

                        let instance_data = instance_data_gather(uri);
                        println!("T{}: {:?}", thread_nr, instance_data);
                    }
                }

                if retry_count >= 5 {
                    break;
                }
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    Ok(())
}

fn instance_data_gather(uri: String) -> Result<Instance, String> {
    let echo_json: serde_json::Value = match reqwest::Client::new()
        .get(&format!("https://{}/api/v1/instance", uri))
        .send() {
        Ok(mut req) => {
            let status: StatusCode = req.status();
            println!("HTTP {}", status);

            let status_code = status.as_u16();
            if status_code >= 400 {
                ()
            }

            match req.json() {
                Ok(data) => {
                    data
                }
                Err(e) => {
                    println!("{}", e);
                    serde_json::from_str(&"{}")?
                    // panic!("crash and burn");
                }
            }
        }
        Err(_e) => {
            panic!("crash and burn");
        }
    };

    println!("Description: {}", echo_json["description"]);
    Ok(Instance {
        uri: echo_json["uri"].to_string(),
        description: echo_json["description"].to_string(),
        email: echo_json["email"].to_string(),
        version: echo_json["version"].to_string(),
        instance_type: "mastodon".to_string(),
    })
}
