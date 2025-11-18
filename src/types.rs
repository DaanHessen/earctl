use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};
use uuid::Uuid;

use crate::models::ModelBase;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BatteryReading {
    Disconnected,
    Level { percent: u8, charging: bool },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatteryStatus {
    pub left: BatteryReading,
    pub right: BatteryReading,
    pub case: BatteryReading,
}

impl BatteryStatus {
    pub fn empty() -> Self {
        Self {
            left: BatteryReading::Disconnected,
            right: BatteryReading::Disconnected,
            case: BatteryReading::Disconnected,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EarSide {
    Left,
    Right,
    Case,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AncLevel {
    Off,
    Transparency,
    NoiseCancellationLow,
    NoiseCancellationHigh,
    NoiseCancellationMid,
    NoiseCancellationAdaptive,
}

impl AncLevel {
    pub fn from_device(value: u8) -> Option<Self> {
        match value {
            0x05 => Some(Self::Off),
            0x07 => Some(Self::Transparency),
            0x03 => Some(Self::NoiseCancellationLow),
            0x01 => Some(Self::NoiseCancellationHigh),
            0x02 => Some(Self::NoiseCancellationMid),
            0x04 => Some(Self::NoiseCancellationAdaptive),
            _ => None,
        }
    }

    pub fn to_device(self) -> u8 {
        match self {
            AncLevel::Off => 0x05,
            AncLevel::Transparency => 0x07,
            AncLevel::NoiseCancellationLow => 0x03,
            AncLevel::NoiseCancellationHigh => 0x01,
            AncLevel::NoiseCancellationMid => 0x02,
            AncLevel::NoiseCancellationAdaptive => 0x04,
        }
    }
}

impl fmt::Display for AncLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            AncLevel::Off => "off",
            AncLevel::Transparency => "transparency",
            AncLevel::NoiseCancellationLow => "nc-low",
            AncLevel::NoiseCancellationHigh => "nc-high",
            AncLevel::NoiseCancellationMid => "nc-mid",
            AncLevel::NoiseCancellationAdaptive => "adaptive",
        };
        write!(f, "{}", label)
    }
}

impl FromStr for AncLevel {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "off" => Ok(AncLevel::Off),
            "transparency" | "transparent" => Ok(AncLevel::Transparency),
            "nc-low" | "low" => Ok(AncLevel::NoiseCancellationLow),
            "nc-high" | "high" => Ok(AncLevel::NoiseCancellationHigh),
            "nc-mid" | "mid" => Ok(AncLevel::NoiseCancellationMid),
            "adaptive" => Ok(AncLevel::NoiseCancellationAdaptive),
            _ => Err("invalid ANC level"),
        }
    }
}

impl fmt::Display for EarSide {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            EarSide::Left => "left",
            EarSide::Right => "right",
            EarSide::Case => "case",
        };
        write!(f, "{}", label)
    }
}

impl FromStr for EarSide {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "left" => Ok(EarSide::Left),
            "right" => Ok(EarSide::Right),
            "case" => Ok(EarSide::Case),
            _ => Err("invalid ear side"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EqMode {
    pub mode: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomEq {
    pub bass: f32,
    pub mid: f32,
    pub treble: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedBassState {
    pub enabled: bool,
    pub level: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalizedAncState {
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyState {
    pub low_latency_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InEarState {
    pub detection_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirmwareInfo {
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EarFitResult {
    pub left: u8,
    pub right: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GestureSlot {
    pub device: u8,
    pub common: u8,
    pub gesture_type: u8,
    pub action: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedColor(pub [u8; 3]);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedColorSet {
    pub pixels: Vec<LedColor>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerialIdentity {
    pub serial_number: Option<String>,
    pub sku: Option<String>,
    pub model_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelSummary {
    pub id: Option<String>,
    pub name: Option<String>,
    pub sku: Option<String>,
    pub serial_number: Option<String>,
    pub base: ModelBase,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub id: Uuid,
    pub port_path: String,
    pub model: Option<ModelSummary>,
}
