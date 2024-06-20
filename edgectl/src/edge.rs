use std::fmt;

use serde::{de::Error, Deserialize, Deserializer, Serialize};
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

#[derive(Debug)]
pub enum InputAdminStatus {
    On,
    Off,
}

impl fmt::Display for InputAdminStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::On => write!(f, "on"),
            Self::Off => write!(f, "off"),
        }
    }
}

impl<'de> Deserialize<'de> for InputAdminStatus {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = u8::deserialize(deserializer)?;
        match value {
            0 => Ok(Self::Off),
            1 => Ok(Self::On),
            _ => Err(D::Error::unknown_variant(&value.to_string(), &["0", "1"])),
        }
    }
}

#[derive(Debug)]
pub enum ThumbnailMode {
    None,
    Core,
}

impl fmt::Display for ThumbnailMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ThumbnailMode::None => write!(f, "none"),
            ThumbnailMode::Core => write!(f, "core"),
        }
    }
}

impl<'de> Deserialize<'de> for ThumbnailMode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = u8::deserialize(deserializer)?;
        match value {
            0 => Ok(ThumbnailMode::None),
            2 => Ok(ThumbnailMode::Core),
            _ => Err(D::Error::unknown_variant(&value.to_string(), &["0", "2"])),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Input {
    pub id: String,
    pub name: String,
    pub admin_status: InputAdminStatus,
    pub owner: String, // group
    // broadcast_standard: String,
    pub buffer_size: u32,
    // channel_group: u8,
    pub created_at: String, // really a date
    pub updated_at: String, // really a date
    pub preview_settings: PreviewSettings,
    pub thumbnail_mode: ThumbnailMode, // 0,1 or 2
    pub tr101290_enabled: bool,
    pub can_subscribe: bool,
    pub appliances: Vec<Appliance>,
    pub health: InputHealth,
    pub ports: Option<Vec<InputPort>>,
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
pub struct InputPort {
    pub id: String,
    pub copies: u8,
    // "internalStreamId": 20485,
    pub physical_port: String,
    // "appliance": "a2253ca5-78a4-45a5-ae96-d8d4091b49ea",
    // "priority": 0,
    pub mode: String,
    // "resolution": "1920x1080",
    // "scanMode": "progressive",
    // "frameRate": "30",
    // "timestampResolution": "seconds"
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Group {
    pub id: String,
    pub name: String,
    pub appliance_secret: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Port {
    pub name: String,
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

    pub fn find_inputs(&self, name: &str) -> Result<Vec<Input>, reqwest::Error> {
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct InputFilter {
            search_name: String,
        }

        #[derive(Serialize)]
        struct EdgeQuery<T: Serialize> {
            filter: T,
        }

        #[derive(Debug, Deserialize)]
        struct InputListResp {
            items: Vec<Input>,
            // total: u32,
        }

        let query = EdgeQuery {
            filter: InputFilter {
                search_name: name.to_owned(),
            },
        };
        let query = serde_json::to_string(&query).expect("Failed to serialize filter as JSON");

        let res = self
            .client
            .get(format!(r#"{}/api/input/?q={}"#, self.url, query))
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

    pub fn get_group(&self, id: &str) -> Result<Group, reqwest::Error> {
        let res = self
            .client
            .get(format!(r#"{}/api/group/{}"#, self.url, id))
            .header("content-type", "application/json")
            .send()?;

        res.json::<Group>()
    }

    pub fn get_port(&self, id: &str) -> Result<Port, reqwest::Error> {
        let res = self
            .client
            .get(format!(r#"{}/api/port/{}"#, self.url, id))
            .header("content-type", "application/json")
            .send()?;

        res.json::<Port>()
    }
}
