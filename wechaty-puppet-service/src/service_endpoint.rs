use serde::Deserialize;
use wechaty_puppet::error::PuppetError;

#[derive(Debug, Deserialize)]
struct Endpoint {
    ip: String,
    port: usize,
}

const WECHATY_ENDPOINT_RESOLUTION_SERVICE_URI: &str = "https://api.chatie.io/v0/hosties/";
const ENDPOINT_SERVICE_ERROR: &str = "Endpoint service error";

pub async fn discover(token: String) -> Result<String, PuppetError> {
    match reqwest::get(&format!("{}{}", WECHATY_ENDPOINT_RESOLUTION_SERVICE_URI, token)).await {
        Ok(res) => match res.json::<Endpoint>().await {
            Ok(endpoint) => {
                if endpoint.port == 0 {
                    Err(PuppetError::InvalidToken)
                } else {
                    Ok(format!("grpc://{}:{}", endpoint.ip, endpoint.port))
                }
            }
            Err(_) => Err(PuppetError::Network(ENDPOINT_SERVICE_ERROR.to_owned())),
        },
        Err(_) => Err(PuppetError::Network(ENDPOINT_SERVICE_ERROR.to_owned())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[actix_rt::test]
    async fn can_discover() {
        println!("{:?}", discover("123".to_owned()).await);
    }
}
