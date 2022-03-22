#[macro_use]
extern crate rocket;
#[macro_use]
extern crate lazy_static;

use mut_static::MutStatic;
use std::ffi::CString;

use rocket::form::Form;
use rocket::response::content::Html;
use rocket::response::{status, Redirect};

mod circular_buffer;
use circular_buffer::CircularBuffer;

const PAGE_SRC: &str = include_str!("page_source.html");

const BUFFER_SIZE: usize = 100;

type MessageBuffer = CircularBuffer<BUFFER_SIZE>;

lazy_static! {
    static ref MSG_BUF: MutStatic<MessageBuffer> = MutStatic::new();
}

#[get("/")]
fn index() -> Html<String> {
    let buf = MSG_BUF.read().unwrap();
    let messages: String = buf
        .into_iter()
        .map(|s| {
            let content: Vec<&str> = s.split("\t").collect();
            format!(
                "<tr> \
            <td>{}</td> \
            <td>{}</td> \
            <td>{}</td> \
        </tr>",
                content[0], content[1], content[2]
            )
        })
        .collect();
    let response = PAGE_SRC.replace("%MESSAGES%", &messages);
    Html(response)
}

#[derive(FromForm)]
struct Message<'r> {
    timestamp: &'r str,
    from: &'r str,
    body: &'r str,
}

impl ToString for Message<'_> {
    fn to_string(&self) -> std::string::String {
        format!("{}\t{}\t{}", self.timestamp, self.from, self.body)
    }
}

#[post("/message", data = "<message>")]
fn message(message: Form<Message<'_>>) -> Redirect {
    let mut mut_buf = MSG_BUF.write().unwrap();
    mut_buf.write_str(&CString::new(message.to_string()).unwrap());
    Redirect::to(uri!(index()))
}

#[launch]
fn rocket() -> _ {
    MSG_BUF.set(MessageBuffer::new()).unwrap();

    rocket::build().mount("/", routes![index, message])
}
