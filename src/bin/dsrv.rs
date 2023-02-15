use actix_web::{error, get, web, App, HttpServer, Result};
use dacal::Dacal;

#[get("/spindles")]
async fn list() -> Result<String> {
    Ok(dacal::devices().unwrap().into_iter()
        .map(|d| d.id.to_string())
        .fold(String::new(), |a,b| a + &b + "\n"))
}

#[get("/spindles/{spindle_id}")]
async fn status(info: web::Path<u16>) -> Result<String> {
    let sid = info.into_inner();

    Dacal::from_id(sid)
        .map(|s| format!("spindle status {} {:?}", s.id, s.get_status()))
        .map_err(|_| error::ErrorNotFound(sid))
}

#[get("/spindles/{spindle_id}/slots/{slot_number}")]
async fn retrieve(info: web::Path<(u16, u8)>) -> Result<String> {
    let (sid, sn) = info.into_inner();

    if let Ok(s) = Dacal::from_id(sid) {
        return s.access_slot(sn)
            .map(|_| "".to_string())
            .map_err(|_| error::ErrorNotFound(format!("{}-{}", sid, sn)));
    }

    Err(error::ErrorNotFound(sid))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    pretty_env_logger::init();

    HttpServer::new(|| {
        App::new()
            .service(list)
            .service(status)
            .service(retrieve)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
