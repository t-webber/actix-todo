```rs
#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello wwworld!")
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}

struct AppState {
    app_name: String,
}

async fn index(data: web::Data<AppState>) -> impl Responder {
    let app_name = &data.app_name;
    format!("<div>a</div> {app_name}!")
}

struct AppStateWithCounter {
    counter: sync::Mutex<i32>, // <- Mutex is necessary to mutate safely across threads
}

#[get("/other")]
async fn other(data: web::Data<AppStateWithCounter>) -> String {
    let mut counter = data.counter.lock().unwrap();
    *counter += 1;

    format!("Request number: {counter}") // <- response with count
}

fn scoped_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/test")
            .route(web::get().to(|| async { HttpResponse::Ok().body("test") }))
            .route(web::head().to(HttpResponse::MethodNotAllowed)),
    );
}

fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/app")
            .route(
                web::get()
                    .to(|| async { HttpResponse::Ok().body("<div style=\"color: red;\">a</div>") }),
            )
            .route(web::head().to(HttpResponse::MethodNotAllowed)),
    );
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    let counter = web::Data::new(AppStateWithCounter {
        counter: sync::Mutex::new(1),
    });
    HttpServer::new(move || {
        let scope = web::scope("/users").service(other);
        App::new()
            .configure(config)
            .service(web::scope("/api").configure(scoped_config))
            .app_data(web::Data::new(AppState {
                app_name: String::from("Actix-web"),
            }))
            .service(
                web::scope("/")
                    .guard(guard::Host("www.rust-lang.org"))
                    .route("", web::to(|| async { HttpResponse::Ok().body("www") })),
            )
            .service(
                web::scope("/")
                    .guard(guard::Host("users.rust-lang.org"))
                    .route("", web::to(|| async { HttpResponse::Ok().body("user") })),
            )
            .app_data(counter.clone())
            .service(hello)
            .service(scope)
            .service(echo)
            .route("/hey", web::get().to(manual_hello))
            .route("/index.html", web::get().to(index))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
```
