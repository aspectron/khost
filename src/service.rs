pub trait Service {
    fn service_name(&self) -> String;
}

pub fn service_name<S: Service>(service: &S) -> String {
    service.service_name()
}
