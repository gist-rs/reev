use reqwest::header::HeaderMap;

pub fn api_client() -> reqwest::Client {
    reqwest::Client::new()
}

pub fn json_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());
    headers.insert("Accept", "application/json".parse().unwrap());
    headers
}
