use crate::imports::*;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize, Hash)]
pub struct ServiceDetail {
    pub caption: String,
    pub name: String,
}

impl ServiceDetail {
    pub fn new<C, N>(caption: C, name: N) -> Self
    where
        C: Display,
        N: Display,
    {
        Self {
            caption: caption.to_string(),
            name: name.to_string(),
        }
    }
}

impl Display for ServiceDetail {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{} ({})", self.caption, self.name)
    }
}

pub trait Service {
    fn service_detail(&self) -> ServiceDetail;
}

pub fn service_name<S: Service>(service: &S) -> String {
    service.service_detail().name
}

pub fn service_detail<S: Service>(service: &S) -> ServiceDetail {
    service.service_detail()
}
