#[macro_use] extern crate rocket;
#[macro_use] extern crate lazy_static;

use mut_static::MutStatic;
use std::ffi::CString;

use rocket::form::Form;
use rocket::response::status;

mod circular_buffer;
use circular_buffer::CircularBuffer;

const BUFFER_SIZE: usize = 100;
const PAGE_SOURCE: &str = include_str!("index.html.template");

type MessageBuffer = CircularBuffer<BUFFER_SIZE>;

lazy_static! {
    static ref MSG_BUF: MutStatic<MessageBuffer> = MutStatic::new();
}

#[get("/")]
fn index() -> String {
    let buf = MSG_BUF.read().unwrap();
    let strings = buf.into_iter().collect::<Vec<String>>();
    strings.join("; ")
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