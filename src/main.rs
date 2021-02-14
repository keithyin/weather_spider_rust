// use http::Uri;
use hyper::body::HttpBody;
use hyper::client::HttpConnector;
use hyper::{Body, Client, Response, Uri};
use serde_json;
use serde_json::Value;
use std::str;
use std::time::{SystemTime, UNIX_EPOCH};
use url::Url;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // This is where we will setup our HTTP client requests.
    let client = Client::new();

    // Parse an `http::Uri`...
    // let uri = "http://www.weather.com.cn/weather/101121401.shtml".parse::<Uri>()?;
    let city_code = get_city_code(&client, "枣庄").await?;
    let target_url = format!(
        "http://www.weather.com.cn/weather1d/{}.shtml#input",
        city_code
    );

    let uri = target_url.parse::<Uri>()?;
    let mut resp = client.get(uri).await?;
    let body = read_response_body(&mut resp).await?;
    println!("{}", body);
    Ok(())
}

fn parse_json(mut body: &str) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
    body = body.trim_start_matches('(');
    body = body.trim_end_matches(')');
    let parsed = serde_json::from_str(body)?;
    Ok(parsed)
}

async fn read_response_body(
    resp: &mut Response<Body>,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let mut body = String::new();

    while let Some(chunk) = resp.body_mut().data().await {
        let part_bytes = &chunk?;
        let part_str = match str::from_utf8(part_bytes) {
            Ok(v) => v,
            Err(_) => {
                // stdout().write_all(part_bytes).await?;
                ""
            }
        };
        body.push_str(part_str);
    }
    Ok(body)
}

async fn get_city_code<'a>(
    client: &'a Client<HttpConnector>,
    cityname: &'a str,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let mut parsed_url = Url::parse("http://toy1.weather.com.cn/search")?;
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?;
    let query = format!("cityname={}&_={}", cityname, timestamp.as_millis());
    parsed_url.set_query(Some(&query));
    let uri = parsed_url.to_string().parse::<Uri>()?;

    // Await the response...
    let mut resp = client.get(uri).await?;

    let body = read_response_body(&mut resp).await?;

    let mut city_code = "";
    let parsed_json = parse_json(&body)?;
    if let Value::Array(ref res) = parsed_json {
        if let Value::Object(ref res) = res[0] {
            if let Value::String(ref res) = res["ref"] {
                let items: Vec<_> = res.split('~').collect();
                city_code = items[0];
            }
        }
    }
    Ok(String::from(city_code))
}
