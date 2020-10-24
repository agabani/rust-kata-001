use crate::routes::status::routes::get;
use actix_web::web::ServiceConfig;

mod models;
mod routes;

pub fn configure(service_config: &mut ServiceConfig) {
    service_config.service(get);
}
