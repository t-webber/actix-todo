#![warn(
    clippy::all,
    clippy::pedantic,
    clippy::style,
    clippy::perf,
    clippy::complexity,
    clippy::correctness,
    clippy::restriction,
    clippy::nursery,
    clippy::cargo
)]
#![allow(
    clippy::missing_docs_in_private_items,
    clippy::blanket_clippy_restriction_lints,
    clippy::cargo_common_metadata
)]
#![allow(
    clippy::single_call_fn,
    clippy::implicit_return,
    clippy::question_mark_used
)]

use actix_web::{
    get, post,
    web::{self, Form},
    App, HttpResponse, HttpServer,
};
use chrono::{Datelike, Timelike};
use core::fmt::Write;
use std::{fs, io, sync};

fn get_html(tasks: &[Task]) -> String {
    let read = fs::read_to_string("lib/main.html").unwrap_or_default();
    let mut split = read.split("@@@").collect::<Vec<_>>();
    let last = split.pop().unwrap_or_default();
    let stringified = tasks.iter().fold(String::new(), |mut acc, task| {
        let title = &task.title;
        let content = &task.content;
        let creation = &task.creation;

        let seconds = &creation.seconds;
        let minutes = &creation.minutes;
        let hours = &creation.hours;
        let days = &creation.days;
        let months = &creation.months;
        let years = &creation.years;

        write!(
            acc,
            "<li style=\"border: 1px solid black; padding: 10px; margin: 10px;\">
            <h2>{title}</h2>
            <p>{content}</p>
            <p>Created at {hours:02}:{minutes:02}:{seconds:02} on {days:02}/{months:02}/{years}.</p>
            </li>",
        )
        .unwrap();
        acc
    });
    format!("{}{stringified}{last}", split.join(""))
}

#[get("/test")]
async fn test(data: web::Data<UserData>) -> HttpResponse {
    match data.tasks.lock() {
        Ok(tasks) => HttpResponse::Ok().body(get_html(&tasks)),
        Err(err) => {
            eprintln!("Error: {err}");
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[derive(serde::Deserialize, Debug)]
struct InputData {
    title_field: String,
    input_field: String,
}

#[post("/add")]
async fn add(form: Form<InputData>, data: web::Data<UserData>) -> HttpResponse {
    let input = form.into_inner();
    match data.tasks.lock() {
        Ok(mut tasks) => {
            tasks.push(Task {
                title: input.title_field,
                content: input.input_field,
                creation: Time::now(),
            });
            HttpResponse::Found()
                .append_header(("Location", "/test"))
                .finish()
        }
        Err(err) => {
            eprintln!("Error: {err}");
            HttpResponse::InternalServerError().finish()
        }
    }
}

struct Time {
    seconds: u32,
    minutes: u32,
    hours: u32,
    days: u32,
    months: u32,
    years: u32,
}

impl Time {
    fn now() -> Self {
        let now = chrono::Local::now();
        let today = now.date_naive();
        Self {
            seconds: now.second(),
            minutes: now.minute(),
            hours: now.hour(),
            days: today.day(),
            months: today.month(),
            years: u32::try_from(today.year().max(0)).unwrap_or_default(),
        }
    }
}

struct Task {
    title: String,
    content: String,
    creation: Time,
}

struct UserData {
    tasks: sync::Mutex<Vec<Task>>,
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    let user_data = web::Data::new(UserData {
        tasks: sync::Mutex::new(vec![]),
    });
    HttpServer::new(move || {
        App::new()
            .app_data(user_data.clone())
            .service(add)
            .service(test)
    })
    .bind(("localhost", 8080))?
    .run()
    .await
}
