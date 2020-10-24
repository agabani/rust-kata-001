mod models;
mod routes;

use actix_web::web::ServiceConfig;
use routes::get;

pub fn configure(service_config: &mut ServiceConfig) {
    service_config.service(get);
}
