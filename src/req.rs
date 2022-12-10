use std::fmt::Write;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct RequestProfile {
    #[serde(with = "http_serde::method", default)]
    pub method: http::method::Method,
    pub url: url::Url,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub params: Option<serde_json::Value>,
    #[serde(
        skip_serializing_if = "http::HeaderMap::is_empty",
        with = "http_serde::header_map",
        default
    )]
    pub headers: http::header::HeaderMap,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub body: Option<serde_json::Value>,
}

#[derive(Debug)]
pub struct ResponseExt(reqwest::Response);

impl ResponseExt {
    pub async fn filter_text(
        self,
        profile: &crate::config::ResponseProfile,
    ) -> anyhow::Result<String> {
        let res = self.0;
        let mut output = String::new();
        writeln!(&mut output, "{:?} {}", res.version(), res.status())?;
        let headers = res.headers();
        for (k, v) in headers.iter() {
            if !profile.skip_headers.iter().any(|sh| sh == k.as_str()) {
                writeln!(&mut output, "{}: {:?}", k, v)?;
            }
        }
        writeln!(&mut output)?;
        let content_type = get_content_type(&headers);
        let text = res.text().await?;
        match content_type.as_deref() {
            Some("application/json") => {
                let text = filter_json(&text, &profile.skip_body)?;
                writeln!(&mut output, "{}", text)?;
            }
            _ => {
                writeln!(&mut output, "{}", text)?;
            }
        }
        Ok(output)
    }
}

fn filter_json(text: &str, skip: &[String]) -> anyhow::Result<String> {
    let mut json = serde_json::from_str::<serde_json::Value>(&text)?;
    match json {
        serde_json::Value::Object(ref mut obj) => {
            for k in skip {
                obj.remove(k);
            }
        }
        // TODO support other type value
        _ => {}
    }
    Ok(serde_json::to_string_pretty(&json)?)
}

impl RequestProfile {
    pub async fn send(&self, args: &crate::ExtraArgs) -> anyhow::Result<ResponseExt> {
        let (headers, query, body) = self.generate(args)?;
        let client = reqwest::Client::new();
        let req = client
            .request(self.method.clone(), self.url.clone())
            .query(&query)
            .headers(headers)
            .body(body)
            .build()?;

        let res = client.execute(req).await?;
        Ok(ResponseExt(res))
    }
    pub fn generate(
        &self,
        args: &crate::ExtraArgs,
    ) -> anyhow::Result<(http::HeaderMap, serde_json::Value, String)> {
        let mut headers = self.headers.clone();
        let mut query = self.params.clone().unwrap_or_else(|| serde_json::json!({}));
        let mut body = self.body.clone().unwrap_or_else(|| serde_json::json!({}));
        for (k, v) in &args.headers {
            headers.insert(k.parse::<http::header::HeaderName>()?, v.parse()?);
        }
        if !headers.contains_key(http::header::CONTENT_TYPE) {
            headers.insert(
                http::header::CONTENT_TYPE,
                http::HeaderValue::from_static("application/json"),
            );
        }
        for (k, v) in &args.query {
            query[k] = v.parse()?;
        }
        for (k, v) in &args.body {
            body[k] = v.parse()?;
        }
        let content_type = get_content_type(&headers);
        match content_type.as_deref() {
            Some("application/json") => {
                let body = serde_json::to_string(&body)?;
                Ok((headers, query, body))
            }
            Some("application/x-www-form-urlencoded" | "multipart/form-data") => {
                let body = serde_urlencoded::to_string(&body)?;
                Ok((headers, query, body))
            }
            _ => Err(anyhow::anyhow!("unsupported content-type")),
        }
    }
}

fn get_content_type(headers: &http::header::HeaderMap) -> Option<String> {
    headers
        .get(http::header::CONTENT_TYPE)
        .map(|v| v.to_str().unwrap().split(';').next())
        .flatten()
        .map(|v| v.to_string())
}
