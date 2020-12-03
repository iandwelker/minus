use tokio::time::sleep;
use futures::join;
use minus::*;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::fmt::Write;

#[tokio::main]
async fn main() {
    let output = Arc::new(Mutex::new(String::new()));
    let increment = async {
        let mut counter: u8 = 0;
        while counter <= 30 {
            let mut output = output.lock().unwrap();
            let _ = writeln!(output, "{}", counter.to_string());
            counter += 1;
            drop(output);
            sleep(Duration::from_millis(100)).await;
        }
    };
    join!(tokio_updating(output.clone()), increment);
}