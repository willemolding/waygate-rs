#[macro_use] extern crate rocket;
#[macro_use] extern crate lazy_static;

use mut_static::MutStatic;
use std::ffi::CString;

use rocket::form::Form;
use rocket::response::status;
use rocket::response::content::Html;

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
    let messages: String = buf.into_iter().map(|s| {
        format!("<tr> \
        <td>?</td> \
        <td>?</td> \
        <td>{}</td> \
      </tr>", s)
    }).collect();
    let response = PAGE_SRC.replace("%MESSAGES%", &messages);
    Html(response)
}

#[derive(FromForm)]
struct Message<'r> {
    // from: &'r str,
    // timestamp: &'r str,
    body: &'r str,
}

#[post("/message", data = "<message>")]
fn message(message: Form<Message<'_>>,) -> status::Accepted<()> {
    let mut mut_buf = MSG_BUF.write().unwrap();
    mut_buf.write_str(&CString::new(message.body).unwrap());
    status::Accepted(Some(()))
}

#[launch]
fn rocket() -> _ {
    MSG_BUF.set(MessageBuffer::new()).unwrap();

    rocket::build()
        .mount("/", routes![index, message])
}