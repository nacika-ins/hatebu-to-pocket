extern crate iron;

use iron::prelude::*;
use iron::status;

fn main() {
    let port = option_env!("PORT").unwrap_or("3000");
    let url = format!("localhost:{}", port);
    Iron::new(|_: &mut Request| Ok(Response::with((status::Ok, "Hello world!"))))
        .http(&*url)
        .unwrap();


}
