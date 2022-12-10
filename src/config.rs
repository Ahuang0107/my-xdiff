use std::io::Write;

use anyhow::Context;

use crate::ExtraArgs;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct DiffConfig {
    #[serde(flatten)]
    pub profiles: std::collections::HashMap<String, DiffProfile>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct DiffProfile {
    pub req1: crate::req::RequestProfile,
    pub req2: crate::req::RequestProfile,
    #[serde(skip_serializing_if = "is_default", default)]
    pub res: ResponseProfile,
}

fn is_default<T: Default + PartialEq>(v: &T) -> bool {
    v == &T::default()
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Default, PartialEq, Eq)]
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
        let config = serde_yaml::from_str::<Self>(content)?;
        config.validate()?;
        Ok(config)
    }
    pub fn get_profile(&self, name: &str) -> Option<&DiffProfile> {
        self.profiles.get(name)
    }
    fn validate(&self) -> anyhow::Result<()> {
        for (name, profile) in &self.profiles {
            profile
                .validate()
                .context(format!("failed to validate profile: {}", name))?;
        }
        Ok(())
    }
}

#[allow(dead_code)]
impl DiffProfile {
    pub async fn diff(&self, args: ExtraArgs) -> anyhow::Result<()> {
        let res1 = self.req1.send(&args).await?;
        let res2 = self.req2.send(&args).await?;
        let text1 = res1.filter_text(&self.res).await?;
        let text2 = res2.filter_text(&self.res).await?;

        let output = crate::utils::diff_text(&text1, &text2)?;
        let stdout = std::io::stdout();
        let mut stdout = stdout.lock();
        write!(stdout, "{}", output)?;
        Ok(())
    }
    fn validate(&self) -> anyhow::Result<()> {
        self.req1.validate().context("req1 failed to validate")?;
        self.req2.validate().context("req2 failed to validate")?;
        Ok(())
    }
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
