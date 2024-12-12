use std::fmt;

use reqwest::blocking::Response;
use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};
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

impl Serialize for InputAdminStatus {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Off => serializer.serialize_u8(0),
            Self::On => serializer.serialize_u8(1),
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

impl Serialize for ThumbnailMode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            ThumbnailMode::None => serializer.serialize_u8(0),
            ThumbnailMode::Core => serializer.serialize_u8(2),
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
    pub appliances: Vec<InputAppliance>,
    pub health: InputHealth,
    pub ports: Option<Vec<InputPort>>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NewInput {
    pub name: String,
    pub tr101290_enabled: bool,
    pub broadcast_standard: String,
    pub thumbnail_mode: ThumbnailMode,
    pub video_preview_mode: String, // "on demand" | "off"
    pub admin_status: InputAdminStatus,
    pub ports: Vec<NewInputPort>,
    pub buffer_size: u16,
    pub max_bitrate: Option<u16>,
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
pub struct InputAppliance {
    pub name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Output {
    pub name: String,
    pub id: String,
    pub admin_status: OutputAdminStatus,
    // The ID of the group
    // pub group: String,
    // The ID of the input
    // pub input: Option<String>,
    pub health: Option<OutputHealth>,
}

#[derive(Debug)]
pub enum OutputAdminStatus {
    Off = 0,
    On = 1,
}

impl<'de> Deserialize<'de> for OutputAdminStatus {
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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutputHealth {
    pub state: OutputHealthState,
    pub title: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum OutputHealthState {
    NotConfigured,
    MetricsMissing,
    Tr101290Priority1Error,
    ReducedRedundancy,
    AllOk,
    NotAcknowledged,
    InputError,
    OutputError,
    Alarm,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Appliance {
    pub name: String,
    pub hostname: String,
    pub contact: String,
    pub serial: String,
    pub id: String,
    pub version: ApplianceVersion,
    // lastMessageAt
    pub last_registered_at: Option<String>, // iso8601/rfc3339
    pub health: Option<ApplianceHealth>,
    pub physical_ports: Vec<AppliancePhysicalPort>,
    // region { id, name }
    #[serde(rename = "type")]
    pub kind: String,
    // owner is the group id
    pub owner: String,
    pub alarms: Vec<ApplianceAlarm>,
    // features
    // logLevel
    // collectHostMetrics
    // ristserverLogLevel
    // settings
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplianceVersion {
    pub control_image_version: String,
    pub control_software_version: String,
    pub data_image_version: String,
    pub data_software_version: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplianceHealth {
    pub title: String,
    pub state: ApplianceHealthState,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ApplianceHealthState {
    Connected,
    Missing,
    NeverConnected,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplianceAlarm {
    pub alarm_cause: String,
    pub alarm_severity: String, // critical | major | minor | warning | cleared
    pub time: String,
    // #[serde(rename = "type")]
    // pub kind: String, // va | edge | backend | backend-monitor | prometheus
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppliancePhysicalPort {
    pub id: String,
    pub name: String,
    // example: 86:71:48:3b:e3:1b
    // pub mac: String,
    // pub index: String,
    // pub portType: enum?,
    // pub appliance: { id, name, type, version }
    // owner is the group id
    // pub owner: Strig,
    pub addresses: Vec<PhysicalPortAddress>,
    // networks: []
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PhysicalPortAddress {
    pub address: String,
    // pub netmask: String,
}

#[derive(Debug, Deserialize, Serialize)]
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

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "mode")]
pub enum NewInputPort {
    Rtp(RtpInputPort),
    Udp(UdpInputPort),
    Sdi(SdiInputPort),
    Generator(GeneratorInputPort),
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RtpInputPort {
    pub copies: u8,
    pub physical_port: String,
    pub address: String,
    pub port: u16,
    pub fec: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub multicast_address: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UdpInputPort {
    pub copies: u8,
    pub physical_port: String,
    pub address: String,
    pub port: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub multicast_address: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SdiInputPort {
    pub copies: u8,
    pub physical_port: String,
    pub encoder_settings: SdiEncoderSettings,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SdiEncoderSettings {
    pub video_codec: String,
    pub total_bitrate: u64,
    pub gop_size_frames: u16,
    pub audio_streams: Vec<SdiEncoderAudioStream>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SdiEncoderAudioStream {
    pub codec: String,
    pub pair: u8,
    pub bitrate: u16,
    #[serde(rename = "type")]
    pub kind: String, // enum: stereo | mono
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneratorInputPort {
    pub copies: u8,
    pub physical_port: String,
    pub bitrate: GeneratorBitrate,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type")]
pub enum GeneratorBitrate {
    Vbr,
    Cbr(GeneratorBitrateCBR),
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneratorBitrateCBR {
    pub bitrate: u64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Group {
    pub id: String,
    pub name: String,
    pub appliance_secret: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Port {
    pub name: String,
}

#[derive(Serialize)]
struct EdgeQuery<T: Serialize> {
    filter: T,
}

#[derive(Debug)]
pub enum EdgeError {
    RequestError(reqwest::Error),
    NonSuccessStatus(reqwest::StatusCode, EdgeApiError),
    ServerError(reqwest::StatusCode, EdgeApiError),
    ClientError(reqwest::StatusCode, EdgeApiError),
}

#[derive(Debug)]
pub enum EdgeApiError {
    ApiError(EdgeErrorResp),
    ParseError(String),
}

#[derive(Deserialize, Debug)]
pub struct EdgeErrorResp {
    title: String,
    detail: EdgeErrorResponseDetail,
    #[serde(rename = "type")]
    kind: String,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum EdgeErrorResponseDetail {
    Detail(String),
    InvalidParameters(Vec<InvalidParameter>),
}

#[derive(Deserialize, Debug)]
struct InvalidParameter {
    name: String,
    reason: String,
}

impl From<Result<EdgeErrorResp, String>> for EdgeApiError {
    fn from(res: Result<EdgeErrorResp, String>) -> Self {
        match res {
            Ok(api_err) => Self::ApiError(api_err),
            Err(e) => Self::ParseError(e),
        }
    }
}

impl std::fmt::Display for EdgeApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::ApiError(api_err) => api_err.fmt(f),
            Self::ParseError(e) => write!(f, "error parsing error: {}", e),
        }
    }
}

impl std::fmt::Display for EdgeErrorResp {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Error: {}, type: {}, details: {}",
            self.title, self.kind, self.detail,
        )
    }
}

impl std::fmt::Display for EdgeErrorResponseDetail {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Detail(s) => s.fmt(f),
            Self::InvalidParameters(params) => write!(
                f,
                "invalid parameters: {}",
                params
                    .iter()
                    .map(|d| format!("{}: {}", d.name, d.reason))
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
        }
    }
}

impl From<reqwest::Error> for EdgeError {
    fn from(item: reqwest::Error) -> Self {
        Self::RequestError(item)
    }
}

impl std::fmt::Display for EdgeError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::RequestError(e) => e.fmt(f),
            Self::NonSuccessStatus(s, res) => {
                write!(f, "Unsuccessful HTTP status ({}): {}", s, res)
            }
            Self::ServerError(s, res) => write!(f, "HTTP server error ({}): {}", s, res),
            Self::ClientError(s, res) => write!(f, "HTTP client error ({}): {}", s, res),
        }
    }
}

trait ResponseExt {
    fn error_if_not_success(self) -> Result<Response, EdgeError>;
}

impl ResponseExt for Response {
    fn error_if_not_success(self) -> Result<Self, EdgeError> {
        let status_code = self.status();
        if !status_code.is_success() {
            let content_type = self
                .headers()
                .get("content-type")
                .and_then(|h| h.to_str().ok())
                .map(|s| s.to_owned());
            let body = self.text().ok();

            let resp = get_edge_error_detail(content_type, body);

            return Err(if status_code.is_client_error() {
                EdgeError::ClientError(status_code, resp.into())
            } else if status_code.is_server_error() {
                EdgeError::ServerError(status_code, resp.into())
            } else {
                EdgeError::NonSuccessStatus(status_code, resp.into())
            });
        }

        Ok(self)
    }
}

fn get_edge_error_detail(
    content_type: Option<String>,
    body: Option<String>,
) -> Result<EdgeErrorResp, String> {
    let Some(body) = body else {
        return Err("Missing body".to_owned());
    };
    let Some(content_type) = content_type else {
        return Err("Failed to decode error response: content-type missing".to_owned());
    };

    if content_type != "application/json" {
        return Err(format!(
            "Decoding error response with content-type {} not supported",
            content_type
        ));
    }

    let json = serde_json::from_str::<EdgeErrorResp>(body.to_owned().as_ref());
    match json {
        Ok(res) => Ok(res),
        Err(e) => Err(format!(
            "Decoding error response as JSON failed, {}, {}",
            e, body
        )),
    }
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

    pub fn create_input(&self, input: NewInput) -> Result<(), EdgeError> {
        self.client
            .post(format!("{}/api/input/", self.url))
            .header("content-type", "application/json")
            .json(&input)
            .send()?
            .error_if_not_success()
            .map(|_| ())
    }

    pub fn delete_input(&self, id: &str) -> Result<(), EdgeError> {
        self.client
            .delete(format!("{}/api/input/{}", self.url, id))
            .send()?
            .error_if_not_success()
            .map(|_| ())
    }

    pub fn list_outputs(&self) -> Result<Vec<Output>, EdgeError> {
        #[derive(Debug, Deserialize)]
        struct OutputListResp {
            items: Vec<Output>,
            // total: u32,
        }

        let res = self
            .client
            .get(format!(r#"{}/api/output/"#, self.url,))
            .header("content-type", "application/json")
            .send()?
            .error_if_not_success()?;

        Ok(res.json::<OutputListResp>()?.items)
    }

    pub fn find_groups(&self, name: &str) -> Result<Vec<Group>, EdgeError> {
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct GroupFilter {
            search_name: String,
        }

        #[derive(Debug, Deserialize)]
        struct GroupListResp {
            items: Vec<Group>,
            // total: u32,
        }

        let query = EdgeQuery {
            filter: GroupFilter {
                search_name: name.to_owned(),
            },
        };
        let query = serde_json::to_string(&query).expect("Failed to serialize filter as JSON");

        let res = self
            .client
            .get(format!(r#"{}/api/group/?q={}"#, self.url, query))
            .header("content-type", "application/json")
            .send()?
            .error_if_not_success()?;

        Ok(res.json::<GroupListResp>()?.items)
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

    pub fn list_appliances(&self) -> Result<Vec<Appliance>, reqwest::Error> {
        #[derive(Debug, Deserialize)]
        struct ApplianceListResponse {
            items: Vec<Appliance>,
            // total: u32,
        }

        let res = self
            .client
            .get(format!(r#"{}/api/appliance/"#, self.url,))
            .header("content-type", "application/json")
            .send()?;

        Ok(res.json::<ApplianceListResponse>()?.items)
    }

    pub fn find_appliances(&self, name: &str) -> Result<Vec<Appliance>, reqwest::Error> {
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct ApplianceFilter {
            search_name: String,
        }

        #[derive(Debug, Deserialize)]
        struct ApplianceListResp {
            items: Vec<Appliance>,
            // total: u32,
        }

        let query = EdgeQuery {
            filter: ApplianceFilter {
                search_name: name.to_owned(),
            },
        };
        let query = serde_json::to_string(&query).expect("Failed to serialize filter as JSON");

        let res = self
            .client
            .get(format!(r#"{}/api/appliance/?q={}"#, self.url, query))
            .header("content-type", "application/json")
            .send()?;

        Ok(res.json::<ApplianceListResp>()?.items)
    }

    pub fn delete_appliance(&self, id: &str) -> Result<(), EdgeError> {
        self.client
            .delete(format!("{}/api/appliance/{}", self.url, id))
            .send()?
            .error_if_not_success()
            .map(|_| ())
    }
}
