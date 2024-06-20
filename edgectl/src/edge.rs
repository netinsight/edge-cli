use serde::Deserialize;
use std::collections::BTreeMap;

pub struct EdgeClient {
    pub client: reqwest::blocking::Client,
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRespUser {
    // id: String,
    // object: String,
    // username: String,
    // role: String,
    // group: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Input {
    pub id: String,
    pub name: String,
    pub admin_status: u8,
    pub owner: String, // group
    // broadcast_standard: String,
    pub buffer_size: u32,
    // channel_group: u8,
    // created_at: String, // really a date
    pub preview_settings: PreviewSettings,
    pub thumbnail_mode: u8, // 0,1 or 2
    pub tr101290_enabled: bool,
    pub can_subscribe: bool,
    pub appliances: Vec<Appliance>,
    pub health: InputHealth,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewSettings {
    pub mode: String, // enum ?
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InputHealth {
    pub state: String,
    pub title: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Appliance {
    pub name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Group {
    pub id: String,
    pub name: String,
    pub appliance_secret: String,
}

impl EdgeClient {
    pub fn with_url(url: &str) -> Self {
        let client = reqwest::blocking::Client::builder()
            .cookie_store(true)
            .build()
            .unwrap();

        Self {
            client,
            url: url.to_owned(),
        }
    }

    pub fn login(
        &self,
        username: String,
        password: String,
    ) -> Result<LoginRespUser, reqwest::Error> {
        let mut login_data = BTreeMap::new();
        login_data.insert("username", username);
        login_data.insert("password", password);

        #[derive(Debug, Deserialize)]
        struct LoginResp {
            user: LoginRespUser,
        }

        let res = self
            .client
            .post(format!("{}/api/login/", self.url))
            .header("content-type", "application/json")
            .json(&login_data)
            .send()?;

        Ok(res.json::<LoginResp>()?.user)
    }

    pub fn list_inputs(&self) -> Result<Vec<Input>, reqwest::Error> {
        #[derive(Debug, Deserialize)]
        struct InputListResp {
            items: Vec<Input>,
            // total: u32,
        }

        let res = self
            .client
            .get(format!(r#"{}/api/input/"#, self.url,))
            .header("content-type", "application/json")
            .send()?;

        Ok(res.json::<InputListResp>()?.items)
    }

    pub fn list_groups(&self) -> Result<Vec<Group>, reqwest::Error> {
        #[derive(Debug, Deserialize)]
        struct GroupListResp {
            items: Vec<Group>,
            // total: u32,
        }

        let res = self
            .client
            .get(format!(r#"{}/api/group/"#, self.url,))
            .header("content-type", "application/json")
            .send()?;

        Ok(res.json::<GroupListResp>()?.items)
    }
}
