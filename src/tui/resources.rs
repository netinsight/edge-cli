use crate::edge::*;
use anyhow::Result;
use ratatui::style::Color;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Strip common unicode symbols (checkmarks, crosses, emojis) from strings
/// to avoid display issues in TUI
fn strip_unicode_symbols(s: &str) -> String {
    s.chars()
        .filter(|c| !matches!(c, '✓' | '✗' | '⚠'))
        .collect::<String>()
        .trim()
        .to_string()
}

/// Output list with its member outputs for display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputListWithOutputs {
    #[serde(flatten)]
    pub list: OutputRecipientList,
    pub outputs: Vec<Output>,
}

/// Group list with its member groups for display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupListWithGroups {
    #[serde(flatten)]
    pub list: GroupRecipientList,
    pub groups: Vec<Group>,
}

#[derive(Debug, Clone)]
pub struct AlarmWithEntities {
    pub alarm: AlarmWithImpact,
    pub input_names: Vec<String>,
    pub output_names: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct AlarmHistoryWithEntities {
    pub alarm: Alarm,
    pub input_names: Vec<String>,
    pub output_names: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceType {
    Input,
    Output,
    OutputList,
    GroupList,
    Appliance,
    Group,
    Region,
    Node,
    Tunnel,
    Settings,
    Alarm,
    AlarmHistory,
}

impl ResourceType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "input" | "inputs" | "i" => Some(Self::Input),
            "output" | "outputs" | "o" => Some(Self::Output),
            "output-list" | "output-lists" | "outputlist" | "outputlists" | "ol" => {
                Some(Self::OutputList)
            }
            "group-list" | "group-lists" | "grouplist" | "grouplists" | "gl" => {
                Some(Self::GroupList)
            }
            "appliance" | "appliances" | "a" => Some(Self::Appliance),
            "group" | "groups" | "g" => Some(Self::Group),
            "region" | "regions" | "r" => Some(Self::Region),
            "node" | "nodes" | "n" => Some(Self::Node),
            "tunnel" | "tunnels" | "t" => Some(Self::Tunnel),
            "settings" | "setting" | "s" => Some(Self::Settings),
            "alarm" | "alarms" | "al" => Some(Self::Alarm),
            "alarm-history" | "ah" => Some(Self::AlarmHistory),
            _ => None,
        }
    }

    pub fn display_name(&self) -> &str {
        match self {
            Self::Input => "Inputs",
            Self::Output => "Outputs",
            Self::OutputList => "Output Lists",
            Self::GroupList => "Group Lists",
            Self::Appliance => "Appliances",
            Self::Group => "Groups",
            Self::Region => "Regions",
            Self::Node => "Nodes",
            Self::Tunnel => "Tunnels",
            Self::Settings => "Settings",
            Self::Alarm => "Active Alarms",
            Self::AlarmHistory => "Alarm History",
        }
    }

    pub fn is_single_item(&self) -> bool {
        matches!(self, Self::Settings)
    }
}

impl fmt::Display for ResourceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceAction {
    Delete,
    Clear,
}

/// A resource item that can be displayed in the TUI
#[derive(Debug, Clone)]
pub enum ResourceItem {
    Input(Input),
    Output(Output),
    OutputList(OutputListWithOutputs),
    GroupList(GroupListWithGroups),
    Appliance(Appliance),
    Group(Group),
    Region(Region),
    Node(KubernetesNode),
    Tunnel(Tunnel),
    Settings(GlobalSettings),
    Alarm(AlarmWithEntities),
    AlarmHistory(AlarmHistoryWithEntities),
}

impl ResourceItem {
    pub fn name(&self) -> String {
        match self {
            Self::Input(i) => i.name.clone(),
            Self::Output(o) => o.name.clone(),
            Self::OutputList(l) => l.list.name.clone(),
            Self::GroupList(l) => l.list.name.clone(),
            Self::Appliance(a) => a.name.clone(),
            Self::Group(g) => g.name.clone(),
            Self::Region(r) => r.name.clone(),
            Self::Node(n) => n.name.clone(),
            Self::Tunnel(t) => format!("{} -> {}", t.client_name, t.server_name),
            Self::Settings(_) => "Global Settings".to_string(),
            Self::Alarm(a) => format!("{} - {}", a.alarm.alarm_severity, a.alarm.alarm_cause),
            Self::AlarmHistory(a) => {
                format!(
                    "{} - {} ({})",
                    a.alarm.alarm_severity, a.alarm.alarm_cause, a.alarm.alarm_id
                )
            }
        }
    }

    /// Get a list of columns for this resource type
    pub fn columns(&self) -> Vec<String> {
        match self {
            Self::Input(_) => vec![
                "Name".to_string(),
                "Status".to_string(),
                "Health".to_string(),
                "Type".to_string(),
            ],
            Self::Output(_) => vec![
                "Name".to_string(),
                "Status".to_string(),
                "Health".to_string(),
                "Type".to_string(),
            ],
            Self::OutputList(_) => vec![
                "Name".to_string(),
                "Members".to_string(),
                "Description".to_string(),
            ],
            Self::GroupList(_) => vec![
                "Name".to_string(),
                "Members".to_string(),
                "Description".to_string(),
            ],
            Self::Appliance(_) => vec![
                "Name".to_string(),
                "Health".to_string(),
                "Version".to_string(),
                "Group".to_string(),
            ],
            Self::Group(_) => vec!["Name".to_string(), "ID".to_string()],
            Self::Region(_) => vec!["Name".to_string(), "Mode".to_string()],
            Self::Node(_) => vec![
                "Name".to_string(),
                "Status".to_string(),
                "Roles".to_string(),
                "Version".to_string(),
            ],
            Self::Tunnel(_) => vec![
                "Source".to_string(),
                "Target".to_string(),
                "Type".to_string(),
                "Status".to_string(),
            ],
            Self::Settings(_) => vec!["Property".to_string(), "Value".to_string()],
            Self::Alarm(_) => vec![
                "Time".to_string(),
                "Severity".to_string(),
                "Cause".to_string(),
                "Message".to_string(),
                "Appliance".to_string(),
                "Entity".to_string(),
            ],
            Self::AlarmHistory(_) => vec![
                "Time".to_string(),
                "ID".to_string(),
                "Severity".to_string(),
                "Cause".to_string(),
                "Message".to_string(),
                "Entity".to_string(),
                "Cleared".to_string(),
            ],
        }
    }

    /// Get row data for this resource
    pub fn row_data(&self) -> Vec<String> {
        match self {
            Self::Input(i) => vec![
                i.name.clone(),
                format!("{:?}", i.admin_status),
                strip_unicode_symbols(&i.health.state),
                i.ports
                    .as_ref()
                    .map(|p| format!("{} ports", p.len()))
                    .unwrap_or_else(|| "No ports".to_string()),
            ],
            Self::Output(o) => vec![
                o.name.clone(),
                format!("{:?}", o.admin_status),
                o.health
                    .as_ref()
                    .map(|h| format!("{:?}", h.state))
                    .unwrap_or_else(|| "Unknown".to_string()),
                format!("{} ports", o.ports.len()),
            ],
            Self::OutputList(l) => vec![
                l.list.name.clone(),
                format!("{} outputs", l.outputs.len()),
                l.list
                    .description
                    .clone()
                    .unwrap_or_else(|| "-".to_string()),
            ],
            Self::GroupList(l) => vec![
                l.list.name.clone(),
                format!("{} groups", l.groups.len()),
                l.list
                    .description
                    .clone()
                    .unwrap_or_else(|| "-".to_string()),
            ],
            Self::Appliance(a) => vec![
                a.name.clone(),
                a.health
                    .as_ref()
                    .map(|h| format!("{:?}", h.state))
                    .unwrap_or_else(|| "Unknown".to_string()),
                a.version.control_software_version.clone(),
                a.owner.clone(),
            ],
            Self::Group(g) => vec![g.name.clone(), g.id.clone()],
            Self::Region(r) => vec![r.name.clone(), format!("{:?}", r.external)],
            Self::Node(n) => vec![
                n.name.clone(),
                n.status.clone(),
                n.roles
                    .iter()
                    .map(|r| format!("{:?}", r))
                    .collect::<Vec<_>>()
                    .join(", "),
                n.kubelet_version
                    .clone()
                    .unwrap_or_else(|| "Unknown".to_string()),
            ],
            Self::Tunnel(t) => vec![
                t.client_name.clone(),
                t.server_name.clone(),
                format!("{:?}", t.r#type),
                format!("{} inputs", t.inputs.len()),
            ],
            Self::Settings(s) => vec!["Log Level".to_string(), format!("{:?}", s.log_level)],
            Self::Alarm(a) => {
                let entities = format_entities(&a.input_names, &a.output_names);
                let time_ago = a
                    .alarm
                    .raised_at
                    .as_ref()
                    .map(|t| format_time_ago(t))
                    .unwrap_or_else(|| "-".to_string());
                let message = truncate_message(a.alarm.text.as_deref().unwrap_or("-"), 50);

                vec![
                    time_ago,
                    a.alarm.alarm_severity.clone(),
                    a.alarm.alarm_cause.clone(),
                    message,
                    a.alarm.appliance_name.as_deref().unwrap_or("-").to_string(),
                    entities,
                ]
            }
            Self::AlarmHistory(a) => {
                let entities = format_entities(&a.input_names, &a.output_names);
                let time_ago = a
                    .alarm
                    .raised_at
                    .as_ref()
                    .map(|t| format_time_ago(t))
                    .unwrap_or_else(|| "-".to_string());
                let message = truncate_message(a.alarm.text.as_deref().unwrap_or("-"), 50);
                let cleared = a
                    .alarm
                    .cleared_at
                    .as_ref()
                    .map(|t| format_time_ago(t))
                    .unwrap_or_else(|| "-".to_string());

                vec![
                    time_ago,
                    a.alarm.alarm_id.clone(),
                    a.alarm.alarm_severity.clone(),
                    a.alarm.alarm_cause.clone(),
                    message,
                    entities,
                    cleared,
                ]
            }
        }
    }

    /// Convert to YAML-like string for describe view
    pub fn to_yaml(&self) -> String {
        let result = match self {
            Self::Input(i) => serde_saphyr::to_string(i),
            Self::Output(o) => serde_saphyr::to_string(o),
            Self::OutputList(l) => serde_saphyr::to_string(l),
            Self::GroupList(l) => serde_saphyr::to_string(l),
            Self::Appliance(a) => serde_saphyr::to_string(a),
            Self::Group(g) => serde_saphyr::to_string(g),
            Self::Region(r) => serde_saphyr::to_string(r),
            Self::Node(n) => serde_saphyr::to_string(n),
            Self::Tunnel(t) => serde_saphyr::to_string(t),
            Self::Settings(s) => serde_saphyr::to_string(s),
            Self::Alarm(a) => serde_saphyr::to_string(&a.alarm),
            Self::AlarmHistory(a) => serde_saphyr::to_string(&a.alarm),
        };

        result.unwrap_or_else(|_| "Failed to serialize to YAML".to_string())
    }

    /// Get the status color for this resource
    /// Blue = healthy, Orange = unconfigured/warning, Red = error/alarm
    pub fn status_color(&self) -> Color {
        match self {
            Self::Input(i) => {
                let state = i.health.state.to_lowercase();
                if state.contains("ok") || state.contains("healthy") || state.contains("connected")
                {
                    Color::Blue
                } else if state.contains("unconfigured")
                    || state.contains("not configured")
                    || state.contains("unknown")
                {
                    Color::Rgb(255, 165, 0) // Orange
                } else if state.contains("error")
                    || state.contains("alarm")
                    || state.contains("failed")
                {
                    Color::Red
                } else {
                    Color::Rgb(255, 165, 0) // Orange for unknown states
                }
            }
            Self::Output(o) => {
                if let Some(ref health) = o.health {
                    match health.state {
                        OutputHealthState::AllOk => Color::Blue,
                        OutputHealthState::ReducedRedundancy => Color::Blue,
                        OutputHealthState::NotConfigured => Color::Rgb(255, 165, 0), // Orange
                        OutputHealthState::MetricsMissing => Color::Rgb(255, 165, 0), // Orange
                        OutputHealthState::NotAcknowledged => Color::Rgb(255, 165, 0), // Orange
                        OutputHealthState::Tr101290Priority1Error => Color::Red,
                        OutputHealthState::InputError => Color::Red,
                        OutputHealthState::OutputError => Color::Red,
                        OutputHealthState::Alarm => Color::Red,
                    }
                } else {
                    Color::Rgb(255, 165, 0) // Orange
                }
            }
            Self::Appliance(a) => {
                if let Some(ref health) = a.health {
                    match health.state {
                        ApplianceHealthState::Connected => Color::Blue,
                        ApplianceHealthState::NeverConnected => Color::Rgb(255, 165, 0), // Orange
                        ApplianceHealthState::Missing => Color::Red,
                    }
                } else {
                    Color::Rgb(255, 165, 0) // Orange
                }
            }
            Self::Node(n) => {
                let status = n.status.to_lowercase();
                if status.contains("ready") && !status.contains("not") {
                    Color::Blue
                } else if status.contains("unknown") || status.contains("pending") {
                    Color::Rgb(255, 165, 0) // Orange
                } else {
                    Color::Red
                }
            }
            Self::Alarm(a) => match a.alarm.alarm_severity.to_lowercase().as_str() {
                "critical" | "major" => Color::Red,
                "minor" => Color::Rgb(255, 165, 0), // Orange
                "warning" => Color::Yellow,
                _ => Color::Blue,
            },
            Self::AlarmHistory(a) => match a.alarm.alarm_severity.to_lowercase().as_str() {
                "critical" | "major" => Color::Red,
                "minor" => Color::Rgb(255, 165, 0), // Orange
                "warning" => Color::Yellow,
                _ => Color::Blue,
            },
            Self::Group(_)
            | Self::Region(_)
            | Self::Tunnel(_)
            | Self::Settings(_)
            | Self::OutputList(_)
            | Self::GroupList(_) => {
                Color::Blue // Default to blue for resources without health status
            }
        }
    }

    /// Returns the action that can be performed on this resource, if any
    pub fn deletable_action(&self) -> Option<ResourceAction> {
        match self {
            // Resources that can be deleted
            Self::Input(_) => Some(ResourceAction::Delete),
            Self::Output(_) => Some(ResourceAction::Delete),
            Self::OutputList(_) => Some(ResourceAction::Delete),
            Self::GroupList(_) => Some(ResourceAction::Delete),
            Self::Appliance(_) => Some(ResourceAction::Delete),
            Self::Group(_) => Some(ResourceAction::Delete),
            Self::Region(_) => Some(ResourceAction::Delete),

            // Resources that can be cleared (not deleted)
            Self::Alarm(_) => Some(ResourceAction::Clear),

            // Resources that cannot be deleted or cleared
            Self::AlarmHistory(_) => None,
            Self::Node(_) => None,
            Self::Tunnel(_) => None,
            Self::Settings(_) => None,
        }
    }
}

/// Fetch all items for a given resource type
pub fn fetch_resources(
    client: &EdgeClient,
    resource_type: ResourceType,
) -> Result<Vec<ResourceItem>> {
    match resource_type {
        ResourceType::Input => {
            let inputs = client.list_inputs()?;
            Ok(inputs.into_iter().map(ResourceItem::Input).collect())
        }
        ResourceType::Output => {
            let outputs = client.list_outputs()?;
            Ok(outputs.into_iter().map(ResourceItem::Output).collect())
        }
        ResourceType::OutputList => {
            let output_lists = client.list_output_recipient_lists()?;
            let mut items = Vec::new();

            for list in output_lists {
                // Fetch member outputs for this list
                let outputs = client.get_output_list_members(&list.id).unwrap_or_default();
                items.push(ResourceItem::OutputList(OutputListWithOutputs {
                    list,
                    outputs,
                }));
            }

            Ok(items)
        }
        ResourceType::GroupList => {
            let group_lists = client.list_group_recipient_lists()?;
            let mut items = Vec::new();

            for list in group_lists {
                // Fetch member groups for this list
                let groups = client.get_group_list_members(&list.id).unwrap_or_default();
                items.push(ResourceItem::GroupList(GroupListWithGroups {
                    list,
                    groups,
                }));
            }

            Ok(items)
        }
        ResourceType::Appliance => {
            let appliances = client.list_appliances()?;
            Ok(appliances
                .into_iter()
                .map(ResourceItem::Appliance)
                .collect())
        }
        ResourceType::Group => {
            let groups = client.list_groups()?;
            Ok(groups.into_iter().map(ResourceItem::Group).collect())
        }
        ResourceType::Region => {
            let regions = client.list_regions()?;
            Ok(regions.into_iter().map(ResourceItem::Region).collect())
        }
        ResourceType::Node => {
            let nodes = client.list_kubernetes_nodes()?;
            Ok(nodes.into_iter().map(ResourceItem::Node).collect())
        }
        ResourceType::Tunnel => {
            let tunnels = client.list_tunnels()?;
            Ok(tunnels.into_iter().map(ResourceItem::Tunnel).collect())
        }
        ResourceType::Settings => {
            let settings = client.global_settings()?;
            Ok(vec![ResourceItem::Settings(settings)])
        }
        ResourceType::Alarm => {
            use std::collections::{HashMap, HashSet};

            let alarms = client.list_alarms()?;

            // Collect all unique input and output IDs
            let mut input_ids = HashSet::new();
            let mut output_ids = HashSet::new();

            for alarm in &alarms {
                if let Some(ref id) = alarm.input_id {
                    input_ids.insert(id.clone());
                }
                if let Some(ref id) = alarm.affected_input {
                    input_ids.insert(id.clone());
                }
                if let Some(ref id) = alarm.output_id {
                    output_ids.insert(id.clone());
                }
                if let Some(ref id) = alarm.affected_output {
                    output_ids.insert(id.clone());
                }
            }

            // Fetch inputs and outputs to build lookup maps
            let input_map: HashMap<String, String> = if !input_ids.is_empty() {
                client
                    .list_inputs()?
                    .into_iter()
                    .filter(|i| input_ids.contains(&i.id))
                    .map(|i| (i.id, i.name))
                    .collect()
            } else {
                HashMap::new()
            };

            let output_map: HashMap<String, String> = if !output_ids.is_empty() {
                client
                    .list_outputs()?
                    .into_iter()
                    .filter(|o| output_ids.contains(&o.id))
                    .map(|o| (o.id, o.name))
                    .collect()
            } else {
                HashMap::new()
            };

            // Build enriched alarms
            let items: Vec<ResourceItem> = alarms
                .into_iter()
                .map(|alarm| {
                    let mut input_names_set = HashSet::new();
                    if let Some(ref id) = alarm.input_id {
                        if let Some(name) = input_map.get(id) {
                            input_names_set.insert(name.clone());
                        }
                    }
                    if let Some(ref id) = alarm.affected_input {
                        if let Some(name) = input_map.get(id) {
                            input_names_set.insert(name.clone());
                        }
                    }
                    if let Some(ref name) = alarm.input_name {
                        input_names_set.insert(name.clone());
                    }

                    let mut output_names_set = HashSet::new();
                    if let Some(ref id) = alarm.output_id {
                        if let Some(name) = output_map.get(id) {
                            output_names_set.insert(name.clone());
                        }
                    }
                    if let Some(ref id) = alarm.affected_output {
                        if let Some(name) = output_map.get(id) {
                            output_names_set.insert(name.clone());
                        }
                    }
                    if let Some(ref name) = alarm.output_name {
                        output_names_set.insert(name.clone());
                    }

                    ResourceItem::Alarm(AlarmWithEntities {
                        alarm,
                        input_names: input_names_set.into_iter().collect(),
                        output_names: output_names_set.into_iter().collect(),
                    })
                })
                .collect();

            Ok(items)
        }
        ResourceType::AlarmHistory => {
            use std::collections::{HashMap, HashSet};

            // Fetch last 100 historical alarms
            let alarms = client.list_alarm_history(100, None, None)?;

            // Collect all unique input and output IDs
            let mut input_ids = HashSet::new();
            let mut output_ids = HashSet::new();

            for alarm in &alarms {
                if let Some(ref id) = alarm.input_id {
                    input_ids.insert(id.clone());
                }
                if let Some(ref id) = alarm.output_id {
                    output_ids.insert(id.clone());
                }
            }

            // Fetch inputs and outputs to build lookup maps
            let input_map: HashMap<String, String> = if !input_ids.is_empty() {
                client
                    .list_inputs()?
                    .into_iter()
                    .filter(|i| input_ids.contains(&i.id))
                    .map(|i| (i.id, i.name))
                    .collect()
            } else {
                HashMap::new()
            };

            let output_map: HashMap<String, String> = if !output_ids.is_empty() {
                client
                    .list_outputs()?
                    .into_iter()
                    .filter(|o| output_ids.contains(&o.id))
                    .map(|o| (o.id, o.name))
                    .collect()
            } else {
                HashMap::new()
            };

            // Build enriched alarm history
            let items: Vec<ResourceItem> = alarms
                .into_iter()
                .map(|alarm| {
                    let mut input_names_set = HashSet::new();
                    if let Some(ref id) = alarm.input_id {
                        if let Some(name) = input_map.get(id) {
                            input_names_set.insert(name.clone());
                        }
                    }
                    if let Some(ref name) = alarm.input_name {
                        input_names_set.insert(name.clone());
                    }

                    let mut output_names_set = HashSet::new();
                    if let Some(ref id) = alarm.output_id {
                        if let Some(name) = output_map.get(id) {
                            output_names_set.insert(name.clone());
                        }
                    }
                    if let Some(ref name) = alarm.output_name {
                        output_names_set.insert(name.clone());
                    }

                    ResourceItem::AlarmHistory(AlarmHistoryWithEntities {
                        alarm,
                        input_names: input_names_set.into_iter().collect(),
                        output_names: output_names_set.into_iter().collect(),
                    })
                })
                .collect();

            Ok(items)
        }
    }
}

/// Delete a resource item
pub fn delete_resource(client: &EdgeClient, item: &ResourceItem) -> Result<()> {
    match item {
        ResourceItem::Input(i) => Ok(client.delete_input(&i.id)?),
        ResourceItem::Output(o) => Ok(client.delete_output(&o.id)?),
        ResourceItem::OutputList(l) => Ok(client.delete_output_recipient_list(&l.list.id)?),
        ResourceItem::GroupList(l) => Ok(client.delete_group_recipient_list(&l.list.id)?),
        ResourceItem::Appliance(a) => Ok(client.delete_appliance(&a.id)?),
        ResourceItem::Group(g) => Ok(client.delete_group(&g.id)?),
        ResourceItem::Region(r) => Ok(client.delete_region(&r.id)?),
        _ => Err(anyhow::anyhow!("Resource type does not support deletion")),
    }
}

/// Clear a resource item (e.g., dismiss an alarm)
pub fn clear_resource(client: &EdgeClient, item: &ResourceItem) -> Result<()> {
    match item {
        ResourceItem::Alarm(a) => {
            let alarm_id = &a.alarm.alarm_id;
            Ok(client.clear_alarm(alarm_id)?)
        }
        _ => Err(anyhow::anyhow!("Resource type does not support clearing")),
    }
}

/// Format time ago (e.g., "2h ago", "15m ago")
fn format_time_ago(time_str: &str) -> String {
    use chrono::{DateTime, Utc};

    let Ok(time) = DateTime::parse_from_rfc3339(time_str) else {
        return time_str.to_string();
    };

    let now = Utc::now();
    let duration = now.signed_duration_since(time.with_timezone(&Utc));

    let seconds = duration.num_seconds();
    if seconds < 60 {
        format!("{}s ago", seconds)
    } else if seconds < 3600 {
        format!("{}m ago", seconds / 60)
    } else if seconds < 86400 {
        format!("{}h ago", seconds / 3600)
    } else {
        format!("{}d ago", seconds / 86400)
    }
}

/// Format entity names for display
fn format_entities(input_names: &[String], output_names: &[String]) -> String {
    let mut entities = Vec::new();
    for name in input_names {
        if !name.is_empty() {
            entities.push(format!("input: {}", name));
        }
    }
    for name in output_names {
        if !name.is_empty() {
            entities.push(format!("output: {}", name));
        }
    }
    if entities.is_empty() {
        "-".to_string()
    } else {
        entities.join(", ")
    }
}

/// Truncate message to specified length
fn truncate_message(msg: &str, max_len: usize) -> String {
    if msg.len() > max_len {
        format!("{}…", &msg[..max_len - 1])
    } else {
        msg.to_string()
    }
}

impl Input {
    pub fn get_all_channels(&self) -> Vec<ChannelInfo> {
        let Some(metrics) = self.metrics.as_ref() else {
            return Vec::new();
        };
        let Some(rist_metrics) = metrics.rist_metrics.as_ref() else {
            return Vec::new();
        };

        rist_metrics
            .iter()
            .filter_map(|metric| {
                if metric.metric_type == "channel" {
                    metric.channel_id.map(|id| ChannelInfo {
                        channel_id: id,
                        active: metric.state.as_deref() == Some("activated"),
                    })
                } else {
                    None
                }
            })
            .collect()
    }
}
