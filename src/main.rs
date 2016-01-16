extern crate iron;

use iron::prelude::*;
use iron::status;

fn main() {
    let port = option_env!("PORT").unwrap_or("3000");
    let url = format!("localhost:{}", port);
    Iron::new(|request: &mut Request| Ok(Response::with((status::Ok, callback(request)))))
        .http(&*url)
        .unwrap();
}


struct Link {
    name: String,
    url: String,
    tags: Vec<String>,
}


fn callback(request: &mut Request) -> String {

    let link = parse_link(request);

    "ok".to_owned()
}


fn parse_link(request: &mut Request) -> Link {

    println!("{:?}", request);

    Link {
        name: "dummy".to_owned(),
        url: "dummy".to_owned(),
        tags: vec![],
    }
}
