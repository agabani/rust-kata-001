mod status;

use actix_web::{web, web::ServiceConfig};

pub fn configure(service_config: &mut ServiceConfig) {
    service_config.service(web::scope("/status").configure(status::configure));
}
