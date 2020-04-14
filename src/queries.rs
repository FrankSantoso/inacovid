use failure::Error;
use reqwest::header;
use url::Url;

pub fn build_headers() -> header::HeaderMap {
    let mut headers = header::HeaderMap::new();
    headers.insert(
        header::USER_AGENT,
        "Mozilla/5.0 (X11; OpenSUSE; Linux x86_64; rv:74.0) Gecko/20100101 Firefox/74.0"
            .parse()
            .expect("Invalid UA"),
    );
    headers.insert(
        "Accept",
        "application/json".parse().expect("Invalid accept type"),
    );
    headers.insert(
        "Accept-Language",
        "en-US,en;q=0.5".parse().expect("Invalid Accept Lang"),
    );
    headers.insert(
        "Origin",
        "https://inacovid19.maps.arcgis.com"
            .parse()
            .expect("Invalid Origin"),
    );
    headers.insert("TE", "Trailers".parse().expect("Invalid TE"));
    headers
}

pub fn make_request_url(uri: &str, qparam: &str) -> Result<String, Error> {
    let mut base_uri = match Url::parse(uri) {
        Ok(u) => u,
        _ => return Err(format_err!("Base URI {} is unrecognized", uri)),
    };
    base_uri.set_query(Some(qparam));
    Ok(base_uri.to_string())
}
