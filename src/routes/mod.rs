mod graph;
mod health;

use actix_web::{web, web::ServiceConfig};

pub(crate) fn configure(service_config: &mut ServiceConfig) {
    service_config
        .service(web::scope("/graph").configure(graph::configure))
        .service(web::scope("/health").configure(health::configure));
}
