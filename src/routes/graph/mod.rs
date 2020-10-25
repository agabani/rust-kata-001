use crate::routes::graph::routes::list;
use actix_web::web::ServiceConfig;

pub mod models;
pub mod routes;

pub fn configure(service_config: &mut ServiceConfig) {
    service_config.service(list);
}
