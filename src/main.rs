extern crate iron;

// iron
use iron::prelude::*;
use iron::status;
use iron::typemap::Key;
extern crate bodyparser;
extern crate persistent;

extern crate router;
use router::Router;

use persistent::Read as perRead;
use persistent::Write as perWrite;

use std::fs::File;
use std::io::prelude::*;

extern crate regex;
use regex::Regex;

extern crate pocket;
use pocket::Pocket;

use std::thread::sleep;
use std::time::Duration;

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


struct PocketMail;
impl Key for PocketMail {
    type Value = String;
}

struct PocketWrap;
impl Key for PocketWrap {
    type Value = Pocket;
}

extern crate toml;


extern crate queryst;
use queryst::parse;

extern crate url;
use url::Url;
use url::percent_encoding::utf8_percent_encode;
use url::percent_encoding::FORM_URLENCODED_ENCODE_SET;

const MAX_BODY_LENGTH: usize = 1024 * 1024 * 10;

fn main() {

    // Load Config
    let mut f = File::open("config.toml").unwrap();
    let mut toml = String::new();
    let _ = f.read_to_string(&mut toml);
    let value = toml::Parser::new(&toml).parse().unwrap();
    let apikey = value.get("hatena")
                      .unwrap()
                      .lookup("apikey")
                      .unwrap()
                      .as_str()
                      .unwrap()
                      .to_owned();
    let consumer_key = value.get("pocket")
                            .unwrap()
                            .lookup("consumer_key")
                            .unwrap()
                            .as_str()
                            .unwrap()
                            .to_owned();
    let redirect_url = value.get("pocket")
                            .unwrap()
                            .lookup("redirect_url")
                            .unwrap()
                            .as_str()
                            .unwrap()
                            .to_owned();
    let pocket_mail = value.get("pocket")
                           .unwrap()
                           .lookup("mail")
                           .unwrap()
                           .as_str()
                           .unwrap()
                           .to_owned();
    let port = option_env!("PORT").unwrap_or("3000");
    let url = format!("localhost:{}", port);

    // Get the access token
    let access_token = {
        let mut f = File::open("pocket.toml");
        if f.is_ok() {
            let mut f = f.unwrap();
            let mut toml = String::new();
            let _ = f.read_to_string(&mut toml);
            let value = toml::Parser::new(&toml).parse().unwrap();
            let access_token = value.get("pocket")
                                    .unwrap()
                                    .lookup("access_token")
                                    .unwrap()
                                    .as_str()
                                    .unwrap()
                                    .to_owned();
            access_token
        } else {
            "".to_owned()
        }
    };

    // Start Server
    let mut router = Router::new();
    router.post("/_/hatena_pocket/", callback);
    router.get("/_/pocket_auth/", auth_pocket);
    let mut chain = Chain::new(router);
    chain.link_before(perRead::<bodyparser::MaxBodyLength>::one(MAX_BODY_LENGTH));
    chain.link_before(perRead::<ApiKey>::one(apikey));
    chain.link_before(perRead::<PocketMail>::one(pocket_mail));

    // Authorize Pocket
    if access_token == "" {
        let mut pocket = Pocket::new(&*consumer_key, None);
        let url = pocket.get_auth_url().unwrap();
        let url = url.to_string().replace("rustapi%3Afinishauth",
                                          &*utf8_percent_encode(&*redirect_url,
                                                                FORM_URLENCODED_ENCODE_SET));
        println!("Follow the link to authorize the app: {}", url);
        chain.link_before(perWrite::<PocketWrap>::one(pocket));
    } else {
        let mut pocket = Pocket::new(&*consumer_key, Some(&*access_token));
        chain.link_before(perWrite::<PocketWrap>::one(pocket));
    }

    Iron::new(chain).http("localhost:3000").unwrap();
}

// Authorize Pocket
fn auth_pocket(request: &mut Request) -> IronResult<Response> {
    let pocket_mail = request.get::<perRead<PocketMail>>().unwrap();
    println!("pocket_mail --> {}", pocket_mail);
    let mut mutex = request.get_mut::<perWrite<PocketWrap>>();
    match mutex {
        Ok(mutex) => {

            let mut pocket = mutex.lock().unwrap();
            let username = pocket.authorize();

            // Authenticate with the agreement of the e-mail address
            if username.is_ok() && pocket_mail.to_string() == username.unwrap() {
                println!("Successful authentication");
                let data = format!("[pocket]\naccess_token = \"{}\"",
                                   pocket.access_token().unwrap());
                let mut f = File::create("pocket.toml").unwrap();
                f.write_all(&data.into_bytes());
                drop(f);

            } else {
                println!("Authentication failure");
            }
        }
        Err(_) => {}
    }

    Ok(Response::with(status::Ok))
}

// Hatena Bookmark Callback
fn callback(request: &mut Request) -> IronResult<Response> {
    let apikey = request.get::<perRead<ApiKey>>().unwrap();
    println!("apikey --> {}", apikey);
    let link = parse_link(request).unwrap();

    if apikey.to_string() == link.apikey {

        println!("--> APIキーが一致しています");

        // Send Pocket
        let mut mutex = request.get_mut::<perWrite<PocketWrap>>();
        match mutex {
            Ok(mutex) => {
                let mut pocket = mutex.lock().unwrap();
                let mut tags = link.tags.clone();
                tags.push("hatenabookmark".to_owned());

                // NOTE: エラーではないのにエラーが返ることがある
                let added_item = pocket.add(&*link.url,
                                            Some(&*link.title),
                                            Some(&*tags.join(",")),
                                            None);
                println!("Pocket投稿結果 --> {:?}", added_item);
            }
            Err(_) => {
                println!("Pocketに投稿が失敗しました");
            }
        }

        // Send Evernote
        // FIXME

        // Send Google Keep
        // FIXME

        // Send Google+
        // FIXME

    }

    Ok(Response::with(status::Ok))
}

// Hatena Bookmark parse Web hook body
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

                                let obj = v.as_object().unwrap();

                                // Api key
                                let key = obj.get("key")
                                             .unwrap()
                                             .as_string()
                                             .unwrap()
                                             .clone()
                                             .to_string();
                                println!("apikey --> {}", key);

                                // Hatena bookmarked url
                                let url = obj.get("url")
                                             .unwrap()
                                             .as_string()
                                             .unwrap()
                                             .clone()
                                             .to_string();
                                println!("url --> {}", url);

                                // title
                                let title = obj.get("title")
                                               .unwrap()
                                               .as_string()
                                               .unwrap()
                                               .clone()
                                               .to_string();
                                println!("title --> {}", title);

                                // username
                                let username = obj.get("username")
                                                  .unwrap()
                                                  .as_string()
                                                  .unwrap()
                                                  .clone()
                                                  .to_string();
                                println!("username --> {}", username);

                                // status
                                let status = obj.get("status")
                                                .unwrap()
                                                .as_string()
                                                .unwrap()
                                                .clone()
                                                .to_string();
                                println!("status --> {}", status);

                                // comment
                                let comment = obj.get("comment")
                                                 .unwrap()
                                                 .as_string()
                                                 .unwrap()
                                                 .clone()
                                                 .to_string();
                                println!("comment --> {}", comment);

                                // tags
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
