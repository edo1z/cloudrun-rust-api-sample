use actix_web::{get, App, Error, HttpRequest, HttpResponse, HttpServer, Responder};
use base64_url;
use futures::future::{ready, Ready};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct User {
    id: String,
    name: String,
    picture: String,
}
impl Responder for User {
    type Error = Error;
    type Future = Ready<Result<HttpResponse, Error>>;

    fn respond_to(self, _req: &HttpRequest) -> Self::Future {
        let body = serde_json::to_string(&self).unwrap();
        ready(Ok(HttpResponse::Ok()
            .content_type("application/json")
            .body(body)))
    }
}

#[get("/hello")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[get("/ping")]
async fn ping() -> impl Responder {
    HttpResponse::Ok().body("pong")
}

#[derive(Serialize, Deserialize, Debug)]
struct UserInfo {
    name: String,
    picture: String,
    user_id: String,
    email: String,
    email_verified: bool,
}

#[get("/profile")]
async fn profile(req: HttpRequest) -> impl Responder {
    let header_value = req.headers().get("X-Endpoint-API-UserInfo").unwrap();
    let b64_url = String::from_utf8(header_value.as_bytes().to_vec()).unwrap();
    let decoded = base64_url::decode(&b64_url).unwrap();
    let serialized = String::from_utf8(decoded).unwrap();
    let user_info: UserInfo = serde_json::from_str(&serialized).unwrap();
    User {
        id: user_info.user_id,
        name: user_info.name,
        picture: user_info.picture,
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(hello).service(ping).service(profile))
        .bind("0.0.0.0:8080")?
        .run()
        .await
}
