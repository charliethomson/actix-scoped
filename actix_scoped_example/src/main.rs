use actix_web::{App, get, HttpResponse, HttpServer, ResponseError};
use actix_web::middleware::TrailingSlash;
use serde::Serialize;
use thiserror::Error;
use uuid::Uuid;
use actix_scoped::{ScopedDefault, ScopedError};
use actix_scoped::*;
use actix_scoped_derive::{HasThreadContext, Scoped};


#[derive(Error, Debug, Clone, Copy)]
pub enum SampleError {
    #[error("Scoped error: {0}")]
    Scoped(#[from] ScopedError)
}

impl ResponseError for SampleError {}

#[derive(Scoped, HasThreadContext, Debug, Clone)]
#[vtype(RequestId)]
#[middleware(error = SampleError)]
pub struct RequestId(Uuid);

impl ScopedDefault for RequestId {
    fn scoped_default() -> Self::Value {
        Self(Uuid::new_v4())
    }
}

struct Response {
    rid: Uuid,
}

#[derive(Serialize)]
struct Response2 {
    data: (Uuid, Uuid, Uuid),
    message: String,
}

fn foo() -> Response {
    Response {
        rid: RequestId::get().unwrap().unwrap().0.clone()
    }
}

#[get("/")]
async fn test(param: RequestId) -> HttpResponse {
    let rid = RequestId::get().unwrap().unwrap().0.clone();
    let prid = param.0.clone();

    let bar = foo();
    let ok = bar.rid == rid && rid == prid;
    let response = Response2 { data: (rid, bar.rid, prid), message: format!("ok?????? {}", ok).into() };
    if !ok {
        return HttpResponse::InternalServerError().json(response);
    }

    return HttpResponse::Ok().json(response);
}

#[actix_web::main]
async fn main() {
    let addr = "0.0.0.0";
    let port = 8080;

    let server = HttpServer::new(move || {
        App::new()
            .wrap(RequestIdMiddlewareFactory)
            .wrap(actix_web::middleware::NormalizePath::new(TrailingSlash::Trim))
            .service(test)
    })
        .bind((addr, port)).unwrap();

    server.run().await.unwrap();
}
