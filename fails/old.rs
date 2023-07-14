use actix_web::{
    cookie::{self, time, Cookie, Key},
    get,
    middleware::Logger,
    web, App, HttpRequest, HttpServer, Responder, Result, HttpResponse,
};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use actix_session::{
    config::PersistentSession, storage::CookieSessionStore, Session, SessionMiddleware,
};
use lazy_static::lazy_static;

lazy_static! {
    static ref SECRET: String = std::env::var("SECRET").unwrap();
}


/// Our claims struct, it needs to derive `Serialize` and/or `Deserialize`
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    aud: String, // Optional. Audience
    exp: usize, // Required (validate_exp defaults to true in validation). Expiration time (as UTC timestamp)
    iat: usize, // Optional. Issued at (as UTC timestamp)
    iss: String, // Optional. Issuer
    nbf: usize, // Optional. Not Before (as UTC timestamp)
    sub: String, // Optional. Subject (whom token refers to)
}

// async fn auth_jwt(session: Session, req: HttpRequest) -> Result<&'static str> {
//     log::info!("{req:?}");

//     // RequestSession trait is used for session access
//     let mut counter = 1;
//     if let Some(count) = session.get::<i32>("counter")? {
//         log::info!("SESSION value: {count}");
//         counter = count + 1;
//         session.insert("counter", counter)?;
//     } else {
//         session.insert("counter", counter)?;
//     }

//     Ok("welcome!")
// }

#[get("/auth/{username}")]
async fn get_jwt(username: web::Path<String>) -> impl Responder {
    let myclaims = Claims {
        aud: "bruh".to_string(),
        exp: time::OffsetDateTime::now_utc().unix_timestamp() as usize + 120,
        iat: 123456789,
        iss: "bruh".to_string(),
        nbf: 123456789,
        sub: username.to_string(),
    };
    let token = encode(
        &Header::default(),
        &myclaims,
        &EncodingKey::from_secret("bruh".as_ref()),
    )
    .unwrap();

    // use the private key to sign the token
    // let token = encode(&Header::new(Algorithm::RS256), &myclaims, &EncodingKey::from_rsa_der(&include_bytes!("../pkey.pem")[..])).unwrap();
    // use the systems ssh key to sign the token .ssh/id_rsa

    let cookie = Cookie::build("jwt", token)
        // .max_age(ActixWebDuration::new(60 * 60, 0))
        .expires(time::OffsetDateTime::now_utc() + time::Duration::days(1))
        .path("/")
        .secure(false)
        .http_only(true)
        .finish();

    // store the cookie in the session
    // let mut session = Session::get_session(&req).unwrap();
    // session.set("jwt", token).unwrap();

    HttpResponse::Ok()
        .cookie(cookie)
        .content_type("text/plain")
        .body("bruh moment")
}

#[get("/verify")]
async fn verify_jwt(req: HttpRequest) -> impl Responder {
    // get cookie from request
    let cookie = req.cookie("jwt").unwrap();
    let token = cookie.value();
    log::info!("{:?}", token);
    // verify the token
    let token_message = decode::<Claims>(&token, &DecodingKey::from_secret("bruh".as_ref()), &Validation::new(Algorithm::HS256));
    match token_message {
        Ok(token) => format!("{:?}", token.claims.sub),
        Err(e) => format!("{:?}", e),
    }
}

#[get("/")]
async fn home() -> impl Responder {
    HttpResponse::NotFound()
        .content_type("text/plain")
        .body("404 not found")
}


#[actix_web::main] // or #[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    log::info!("starting HTTP server at http://localhost:8080");

    HttpServer::new(|| {
        App::new()
            // enable logger
            .wrap(Logger::default())
            // cookie session middleware
            .wrap(
                SessionMiddleware::builder(CookieSessionStore::default(), Key::from(&[0; 64]))
                    .cookie_secure(false)
                    // customize session and cookie expiration
                    .session_lifecycle(
                        PersistentSession::default().session_ttl(cookie::time::Duration::hours(24)),
                    )
                    .build(),
            )
            .service(get_jwt)
            // .service(web::resource("/").to(auth_jwt))
            .service(verify_jwt)
            .service(home)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
