extern crate iron;

// iron
use iron::prelude::*;
use iron::status;
use iron::typemap::Key;
extern crate bodyparser;
extern crate persistent;

use persistent::Read as perRead;

use std::fs::File;
use std::io::prelude::*;

extern crate regex;
use regex::Regex;

struct Link {
    url: String,
    tags: Vec<String>,
    apikey: String,
    comment: String,
    status: String,
    title: String,
    username: String,
}

struct ApiKey;
impl Key for ApiKey {
    type Value = String;
}


extern crate toml;


extern crate queryst;
use queryst::parse;

const MAX_BODY_LENGTH: usize = 1024 * 1024 * 10;

fn main() {

    let mut f = File::open("config.toml").unwrap();
    let mut toml = String::new();
    let _ = f.read_to_string(&mut toml);
    let value = toml::Parser::new(&toml).parse().unwrap();
    let apikey = value.get("apikey")
                      .unwrap()
                      .lookup("apikey")
                      .unwrap()
                      .as_str()
                      .unwrap()
                      .to_owned();

    let port = option_env!("PORT").unwrap_or("3000");
    let url = format!("localhost:{}", port);

    // Server Start
    // Iron::new(|request: &mut Request| {
    //     Ok(Response::with((status::Ok, callback(request, apikey.clone()))))
    // })
    //     .http(&*url)
    //     .unwrap();

    let mut chain = Chain::new(callback);
    chain.link_before(perRead::<bodyparser::MaxBodyLength>::one(MAX_BODY_LENGTH));
    chain.link_before(perRead::<ApiKey>::one(apikey));
    Iron::new(chain).http("localhost:3000").unwrap();
}


fn callback(request: &mut Request) -> IronResult<Response> {
    let apikey = request.get::<perRead<ApiKey>>().unwrap();
    println!("apikey --> {}", apikey);
    let link = parse_link(request).unwrap();

    if apikey.to_string() == link.apikey {

        println!("--> APIキーが一致しています");

        // Send Pocket
        // FIXME

        // Send Evernote
        // FIXME

        // Send Google Keep
        // FIXME

        // Send Google+
        // FIXME

    }

    Ok(Response::with(status::Ok))
}


fn parse_link(request: &mut Request) -> Option<Link> {
    // println!("{:?}", request);

    let body = request.get::<bodyparser::Raw>();
    // println!("{:?}", body);

    match body {
        Ok(v) => {
            match v {
                Some(v) => {
                    let object = parse(&*v);
                    match object {
                        Ok(v) => {
                            println!("{:?}", v);
                            if v.is_object() {

                                println!("------------------------------------------------");
                                let obj = v.as_object().unwrap();

                                // APIキー
                                let key = obj.get("key")
                                             .unwrap()
                                             .as_string()
                                             .unwrap()
                                             .clone()
                                             .to_string();
                                println!("apikey --> {}", key);

                                // ハテブされたURL
                                let url = obj.get("url")
                                             .unwrap()
                                             .as_string()
                                             .unwrap()
                                             .clone()
                                             .to_string();
                                println!("url --> {}", url);

                                // タイトル
                                let title = obj.get("title")
                                               .unwrap()
                                               .as_string()
                                               .unwrap()
                                               .clone()
                                               .to_string();
                                println!("title --> {}", title);

                                // ユーザー名
                                let username = obj.get("username")
                                                  .unwrap()
                                                  .as_string()
                                                  .unwrap()
                                                  .clone()
                                                  .to_string();
                                println!("username --> {}", username);

                                // ステータス
                                let status = obj.get("status")
                                                .unwrap()
                                                .as_string()
                                                .unwrap()
                                                .clone()
                                                .to_string();
                                println!("status --> {}", status);

                                // コメント
                                let comment = obj.get("comment")
                                                 .unwrap()
                                                 .as_string()
                                                 .unwrap()
                                                 .clone()
                                                 .to_string();
                                println!("comment --> {}", comment);

                                // タグ
                                let mut tags: Vec<String> = Vec::new();
                                let re = Regex::new(r"\[([^\]]+)\]").unwrap();
                                for cap in re.captures_iter(&*comment) {
                                    println!("cap --> {:?}", cap.at(1));
                                    match cap.at(1) {
                                        Some(v) => {
                                            tags.push(v.to_string());
                                        }
                                        None => (),
                                    }
                                }

                                Some(Link {
                                    url: url,
                                    tags: tags,
                                    apikey: key,
                                    title: title,
                                    status: status,
                                    comment: comment,
                                    username: username,
                                })

                            } else {
                                None
                            }
                        }
                        Err(v) => None,
                    }
                }
                None => None,
            }
        }
        Err(_) => None,
    }

}
