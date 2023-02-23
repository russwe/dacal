use actix_web::{error, get, web, App, HttpServer, Result};
use dacal::{Dacal, error::SpindleError};
use clap::Parser;

// sudo setcap cap_net_bind_service=ep ~/dsrv  ;_; but it works

#[derive(Parser)]
struct Cli {
    #[arg(short, long)]
    bind: Option<String>,
}

#[get("/spindles")]
async fn list() -> Result<String> {
    Ok(dacal::devices()
        .map(|v| v.into_iter()
            .map(|s| s.id.to_string())
            .fold(String::new(), |a,b| a + &b + "\n"))
        .map_err(|e| error::ErrorInternalServerError(e))?)
}

#[get("/spindles/{spindle_id}")]
async fn status(info: web::Path<u16>) -> Result<String> {
    let sid = info.into_inner();

    do_dacal(sid, |s| {
        Ok(format!("{}: {}", sid, s.get_status()?))
    })
}

#[get("/spindles/{spindle_id}/slots")]
async fn identify(info: web::Path<u16>) -> Result<String> {
    let sid = info.into_inner();

    do_dacal(sid, |s| {
        s.set_led(true)?;
        Ok(format!("Lighting {}", sid))
    })
}

#[get("/spindles/{spindle_id}/slots/{slot_number}")]
async fn retrieve(info: web::Path<(u16, u8)>) -> Result<String> {
    let (sid, sn) = info.into_inner();

    do_dacal(sid, |s| {
        s.access_slot(sn)?;
        Ok(format!("Accessing {}-{}", sid, sn))
    })
}

fn do_dacal<F: FnOnce(Dacal) -> std::result::Result<String, SpindleError>>(sid: u16, cmds: F) -> Result<String> {
    if let Ok(dacal) = Dacal::from_id(sid) {
        cmds(dacal).map_err(|e| error::ErrorInternalServerError(e))
    } else {
        Err(error::ErrorNotFound(format!("{}", sid)))
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    pretty_env_logger::init();

    let cli = Cli::parse();
    let ip_port = cli.bind.unwrap_or("0.0.0.0:8080".to_string());

    HttpServer::new(|| {
        App::new()
            .service(list)
            .service(identify)
            .service(status)
            .service(retrieve)
    })
    .bind(&ip_port)?
    .run()
    .await
}
