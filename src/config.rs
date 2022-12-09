#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct DiffConfig {
    #[serde(flatten)]
    pub profiles: std::collections::HashMap<String, DiffProfile>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct DiffProfile {
    pub req1: RequestProfile,
    pub req2: RequestProfile,
    pub res: ResponseProfile,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct RequestProfile {
    #[serde(with = "http_serde::method")]
    pub method: http::method::Method,
    pub url: url::Url,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub params: Option<serde_json::Value>,
    #[serde(
        skip_serializing_if = "http::HeaderMap::is_empty",
        with = "http_serde::header_map",
        default
    )]
    pub headers: http::HeaderMap,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub body: Option<serde_json::Value>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ResponseProfile {
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub skip_headers: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub skip_body: Vec<String>,
}

#[allow(dead_code)]
impl DiffConfig {
    pub async fn load_yaml(path: &str) -> anyhow::Result<Self> {
        let content = tokio::fs::read_to_string(path).await?;
        Self::from_yaml(&content)
    }
    pub fn from_yaml(content: &str) -> anyhow::Result<Self> {
        Ok(serde_yaml::from_str(content)?)
    }
    pub fn get_profile(&self, name: &str) -> Option<&DiffProfile> {
        self.profiles.get(name)
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct DiffArgs {}

#[allow(dead_code)]
impl DiffProfile {
    pub async fn diff(&self, args: DiffArgs) -> anyhow::Result<String> {
        let res1 = self.req1.send(&args).await?;
        let res2 = self.req2.send(&args).await?;
        let text1 = res1.filter_text(&self.res).await?;
        let text2 = res2.filter_text(&self.res).await?;
        Ok(text_diff(text1, text2))
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct WrapResponse {}

impl RequestProfile {
    pub async fn send(&self, args: &DiffArgs) -> anyhow::Result<WrapResponse> {
        todo!()
    }
}

impl WrapResponse {
    pub async fn filter_text(&self, res: &ResponseProfile) -> anyhow::Result<&str> {
        todo!()
    }
}

pub fn text_diff(t1: &str, t2: &str) -> String {
    todo!()
}

#[cfg(test)]
mod test {
    #[tokio::test]
    async fn it_works() -> anyhow::Result<()> {
        let config = crate::config::DiffConfig::load_yaml("./fixtures/test.yml").await?;
        println!("{:#?}", config);
        assert_eq!(config.profiles.len(), 1);
        assert!(config.get_profile("rust").is_some());
        assert!(config.get_profile("other").is_none());
        Ok(())
    }
}
