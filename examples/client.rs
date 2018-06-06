#![deny(warnings)]
extern crate futures;
extern crate tokio;
extern crate hyper;
extern crate pretty_env_logger;

use std::env;
use std::io::{self, Write};

use futures::{Future, Stream};
use tokio::runtime::Runtime;

use hyper::Client;

fn main() {
    pretty_env_logger::init();

    // Some simple CLI args requirements...
    let url = match env::args().nth(1) {
        Some(url) => url,
        None => {
            "http://www.columbia.edu/~fdc/sample.html".to_owned()
        }
    };

    // HTTPS requires picking a TLS implementation, so give a better
    // warning if the user tries to request an 'https' URL.
    let url = url.parse::<hyper::Uri>().unwrap();
    if url.scheme_part().map(|s| s.as_ref()) != Some("http") {
        println!("This example only works with 'http' URLs.");
        return;
    }

    let mut runtime = Runtime::new().unwrap();
    let client = Client::new();

    let job = client
    // Fetch the url...
        .get(url)
    // And then, if we get a response back...
        .and_then(|res| {
            println!("Response: {}", res.status());
            println!("Headers: {:#?}", res.headers());

            // The body is a stream, and for_each returns a new Future
            // when the stream is finished, and calls the closure on
            // each chunk of the body...
            res.into_body().for_each(|chunk| {
                io::stdout().write_all(&chunk)
                    .map_err(|e| panic!("example expects stdout is open, error={}", e))
            })
        })
    // If all good, just tell the user...
        .map(|_| {
            println!("\n\nDone.");
        })
    // If there was an error, let the user know...
        .map_err(|err| {
            eprintln!("Error {}", err);
        });

    runtime.spawn(job); // non-blocking
    drop(client); // below shutdown would halt without this drop.
    runtime.shutdown_on_idle().wait().unwrap();
}
