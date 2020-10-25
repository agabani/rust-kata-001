mod models;
mod routes;

use actix_web::web::ServiceConfig;
use routes::get;

pub(crate) fn configure(service_config: &mut ServiceConfig) {
    service_config.service(get);
}
