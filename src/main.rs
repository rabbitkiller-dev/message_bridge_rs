use serde::Deserialize;
use serde::Serialize;

use mirai_rs::Mirai;

#[derive(Deserialize, Serialize, Debug, Eq, PartialEq)]
pub struct AboutResponse {
    pub code: u32,
    pub data: AboutData,
}
#[derive(Deserialize, Serialize, Debug, Eq, PartialEq)]
pub struct AboutData {
    pub version: String,
}
pub type HttpResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;
//
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mirai = Mirai::new("http://52.193.15.252", 8080, "INITKEYKPRGCLwL");

    Ok(())
}

async fn about() -> HttpResult<AboutResponse> {
    let client = reqwest::Client::new();
    let resp: AboutResponse = client
        .get("http://52.193.15.252:8080/about")
        .send()
        .await?
        .json()
        .await?;

    Ok(resp)
}

#[cfg(test)]
#[allow(non_snake_case)]
mod test {
    use super::*;
    macro_rules! aw {
        ($e:expr) => {
            tokio_test::block_on($e)
        };
    }

    #[test]
    fn test() -> Result<(), Box<dyn std::error::Error>> {
        let mut mirai =
            Mirai::new("http://52.193.15.252", 8080, "INITKEYKPRGCLwL").bind_qq(3245538509);
        let resp = tokio_test::block_on(mirai.verify());
        println!("{:?}", resp);
        let resp = tokio_test::block_on(mirai.bind());

        println!("{:?}", resp);

        Ok(())
    }
}
