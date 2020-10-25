use crate::routes::graph::routes::list;
use actix_web::web::ServiceConfig;

pub(crate) mod models;
pub(crate) mod routes;

pub(crate) fn configure(service_config: &mut ServiceConfig) {
    service_config.service(list);
}
