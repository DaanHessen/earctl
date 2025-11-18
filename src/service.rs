use std::sync::Arc;

use tokio::sync::{Mutex, RwLock};
use uuid::Uuid;

use crate::{
    connection::EarConnection,
    error::EarError,
    models::{ModelBase, model_from_id, model_from_sku},
    protocol::{command, response},
    types::{
        AncLevel, BatteryReading, BatteryStatus, CustomEq, EarFitResult, EarSide,
        EnhancedBassState, EqMode, FirmwareInfo, GestureSlot, InEarState, LatencyState, LedColor,
        LedColorSet, ModelSummary, PersonalizedAncState, SerialIdentity, SessionInfo,
    },
};

pub struct EarManager {
    session: RwLock<Option<Arc<EarSession>>>,
}

impl EarManager {
    pub fn new() -> Self {
        Self {
            session: RwLock::new(None),
        }
    }

    pub async fn connect(
        &self,
        address: bluer::Address,
        channel: u8,
    ) -> Result<EarSessionHandle, EarError> {
        let mut guard = self.session.write().await;
        if guard.is_some() {
            return Err(EarError::AlreadyConnected);
        }

        let connection = EarConnection::open(address, channel).await?;
        let port_path = connection.port_path().to_string();

        tracing::info!("Connected to RFCOMM {}", port_path);

        let session = Arc::new(EarSession {
            id: Uuid::new_v4(),
            port_path,
            connection: Mutex::new(connection),
            model: RwLock::new(None),
        });
        let handle = EarSessionHandle {
            inner: session.clone(),
        };
        *guard = Some(session);

        Ok(handle)
    }

    pub async fn session(&self) -> Result<EarSessionHandle, EarError> {
        let guard = self.session.read().await;
        guard
            .as_ref()
            .cloned()
            .map(|inner| EarSessionHandle { inner })
            .ok_or(EarError::NoSession)
    }

    pub async fn disconnect(&self) -> Result<(), EarError> {
        let mut guard = self.session.write().await;
        if guard.is_none() {
            return Err(EarError::NoSession);
        }
        *guard = None;
        Ok(())
    }
}

#[derive(Clone)]
pub struct EarSessionHandle {
    inner: Arc<EarSession>,
}

struct EarSession {
    id: Uuid,
    port_path: String,
    connection: Mutex<EarConnection>,
    model: RwLock<Option<ModelDescriptor>>,
}

#[derive(Clone)]
struct ModelDescriptor {
    base: ModelBase,
    model_id: Option<String>,
    name: Option<String>,
    sku: Option<String>,
    serial: Option<String>,
}

impl ModelDescriptor {
    fn summary(&self) -> ModelSummary {
        ModelSummary {
            id: self.model_id.clone(),
            name: self.name.clone(),
            sku: self.sku.clone(),
            serial_number: self.serial.clone(),
            base: self.base,
        }
    }
}

impl Default for ModelDescriptor {
    fn default() -> Self {
        Self {
            base: ModelBase::Unknown,
            model_id: None,
            name: None,
            sku: None,
            serial: None,
        }
    }
}

impl EarSessionHandle {
    pub fn id(&self) -> Uuid {
        self.inner.id
    }

    pub async fn info(&self) -> SessionInfo {
        let model = self.inner.model.read().await.clone().map(|m| m.summary());
        SessionInfo {
            id: self.inner.id,
            port_path: self.inner.port_path.clone(),
            model,
        }
    }

    pub async fn set_model_by_id(&self, id: &str) -> Result<ModelSummary, EarError> {
        let info = model_from_id(id).ok_or(EarError::UnknownModel)?;
        let descriptor = ModelDescriptor {
            base: info.base,
            model_id: Some(info.id.to_string()),
            name: Some(info.name.to_string()),
            sku: None,
            serial: None,
        };
        *self.inner.model.write().await = Some(descriptor.clone());
        Ok(descriptor.summary())
    }

    pub async fn set_model_base(&self, base: ModelBase) -> ModelSummary {
        let descriptor = ModelDescriptor {
            base,
            model_id: None,
            name: None,
            sku: None,
            serial: None,
        };
        *self.inner.model.write().await = Some(descriptor.clone());
        descriptor.summary()
    }

    pub async fn set_model_from_sku(
        &self,
        sku: &str,
        serial: Option<String>,
    ) -> Result<ModelSummary, EarError> {
        let info = model_from_sku(sku).ok_or(EarError::UnknownModel)?;
        let descriptor = ModelDescriptor {
            base: info.base,
            model_id: Some(info.id.to_string()),
            name: Some(info.name.to_string()),
            sku: Some(sku.to_string()),
            serial,
        };
        *self.inner.model.write().await = Some(descriptor.clone());
        Ok(descriptor.summary())
    }

    /// Initialize device by querying all its states (like ear-web's initDevice)
    pub async fn init_device(&self) -> Result<(), EarError> {
        use tokio::time::{Duration, sleep};

        tracing::debug!("Starting device initialization...");

        // Request battery
        let _ = self.read_battery().await;
        sleep(Duration::from_millis(100)).await;

        // Request EQ
        let _ = self.read_eq().await;
        sleep(Duration::from_millis(100)).await;

        // Request in-ear status
        let _ = self.read_in_ear().await;
        sleep(Duration::from_millis(100)).await;

        // Request latency status
        let _ = self.read_latency().await;
        sleep(Duration::from_millis(100)).await;

        tracing::debug!("Device initialization complete");
        Ok(())
    }

    pub async fn detect_serial(&self) -> Result<SerialIdentity, EarError> {
        let payload = {
            let conn = self.inner.connection.lock().await;
            conn.transact(
                command::REQUEST_SERIAL,
                &[],
                |packet| {
                    if packet.command == response::SERIAL {
                        Some(packet.payload.clone())
                    } else {
                        None
                    }
                },
                "serial",
            )
            .await?
        };

        let serial = parse_serial_number(&payload);
        let mut sku = None;
        let mut model_summary = None;
        if let Some(ref serial_number) = serial {
            if let Some(detected_sku) = derive_sku_from_serial(serial_number) {
                if let Some(info) = model_from_sku(detected_sku.as_str()) {
                    model_summary = Some(info);
                }
                sku = Some(detected_sku);
            }
        }

        if let Some(info) = model_summary {
            let descriptor = ModelDescriptor {
                base: info.base,
                model_id: Some(info.id.to_string()),
                name: Some(info.name.to_string()),
                sku: sku.clone(),
                serial: serial.clone(),
            };
            *self.inner.model.write().await = Some(descriptor);
        }

        Ok(SerialIdentity {
            serial_number: serial,
            sku,
            model_id: model_summary.map(|info| info.id.to_string()),
        })
    }

    pub async fn read_battery(&self) -> Result<BatteryStatus, EarError> {
        let conn = self.inner.connection.lock().await;
        conn.transact(
            command::REQUEST_BATTERY,
            &[],
            |packet| match packet.command {
                response::BATTERY_PRIMARY | response::BATTERY_SECONDARY => {
                    Some(parse_battery_payload(&packet.payload))
                }
                _ => None,
            },
            "battery",
        )
        .await
    }

    pub async fn read_anc(&self) -> Result<AncLevel, EarError> {
        self.require_support("ANC read", |base| base != ModelBase::B157)
            .await?;
        let conn = self.inner.connection.lock().await;
        conn.transact(
            command::REQUEST_ANC,
            &[],
            |packet| match packet.command {
                response::ANC_PRIMARY | response::ANC_SECONDARY => packet
                    .payload
                    .get(1)
                    .and_then(|&value| AncLevel::from_device(value)),
                _ => None,
            },
            "anc",
        )
        .await
    }

    pub async fn set_anc(&self, level: AncLevel) -> Result<(), EarError> {
        self.require_support("ANC write", |base| base != ModelBase::B157)
            .await?;
        let conn = self.inner.connection.lock().await;
        let mut payload = [0x01u8, 0x01, 0x00];
        payload[1] = level.to_device();
        conn.send_command(command::CMD_SET_ANC, &payload).await?;
        Ok(())
    }

    pub async fn read_eq(&self) -> Result<EqMode, EarError> {
        let conn = self.inner.connection.lock().await;
        conn.transact(
            command::REQUEST_EQ,
            &[],
            |packet| match packet.command {
                response::EQ_PRIMARY | response::EQ_LISTENING_MODE => {
                    packet.payload.first().copied().map(|mode| EqMode { mode })
                }
                _ => None,
            },
            "eq",
        )
        .await
    }

    pub async fn set_eq_mode(&self, mode: u8) -> Result<(), EarError> {
        let conn = self.inner.connection.lock().await;
        conn.send_command(command::CMD_SET_EQ, &[mode, 0x00])
            .await?;
        Ok(())
    }

    pub async fn get_custom_eq(&self) -> Result<CustomEq, EarError> {
        self.require_support("custom EQ", |base| base.supports_custom_eq())
            .await?;
        let conn = self.inner.connection.lock().await;
        conn.transact(
            command::REQUEST_CUSTOM_EQ,
            &[],
            |packet| {
                if packet.command == response::CUSTOM_EQ {
                    decode_custom_eq(&packet.payload)
                } else {
                    None
                }
            },
            "custom_eq",
        )
        .await
    }

    pub async fn set_custom_eq(&self, eq: CustomEq) -> Result<(), EarError> {
        self.require_support("custom EQ", |base| base.supports_custom_eq())
            .await?;
        let conn = self.inner.connection.lock().await;
        let payload = encode_custom_eq(eq);
        conn.send_command(command::CMD_SET_CUSTOM_EQ, &payload)
            .await?;
        Ok(())
    }

    pub async fn read_enhanced_bass(&self) -> Result<EnhancedBassState, EarError> {
        self.require_support("enhanced bass", |base| base.supports_enhanced_bass())
            .await?;
        let conn = self.inner.connection.lock().await;
        conn.transact(
            command::REQUEST_ENHANCED_BASS,
            &[],
            |packet| {
                if packet.command == response::ENHANCED_BASS {
                    let enabled = packet.payload.get(0).copied().unwrap_or_default() > 0;
                    let level = packet.payload.get(1).copied().unwrap_or_default() / 2;
                    Some(EnhancedBassState { enabled, level })
                } else {
                    None
                }
            },
            "enhanced_bass",
        )
        .await
    }

    pub async fn set_enhanced_bass(&self, enabled: bool, level: u8) -> Result<(), EarError> {
        self.require_support("enhanced bass", |base| base.supports_enhanced_bass())
            .await?;
        let conn = self.inner.connection.lock().await;
        let mut payload = [0u8, 0u8];
        if enabled {
            payload[0] = 0x01;
        }
        payload[1] = level.saturating_mul(2);
        conn.send_command(command::CMD_SET_ENHANCED_BASS, &payload)
            .await?;
        Ok(())
    }

    pub async fn get_personalized_anc(&self) -> Result<PersonalizedAncState, EarError> {
        self.require_support("personalized ANC", |base| base.supports_personalized_anc())
            .await?;
        let conn = self.inner.connection.lock().await;
        conn.transact(
            command::REQUEST_PERSONALIZED_ANC,
            &[],
            |packet| {
                if packet.command == response::PERSONALIZED_ANC {
                    packet.payload.first().map(|&value| PersonalizedAncState {
                        enabled: value == 1,
                    })
                } else {
                    None
                }
            },
            "personalized_anc",
        )
        .await
    }

    pub async fn set_personalized_anc(&self, enabled: bool) -> Result<(), EarError> {
        self.require_support("personalized ANC", |base| base.supports_personalized_anc())
            .await?;
        let conn = self.inner.connection.lock().await;
        let value = if enabled { 0x01 } else { 0x00 };
        conn.send_command(command::CMD_SET_PERSONALIZED_ANC, &[value])
            .await?;
        Ok(())
    }

    pub async fn read_in_ear(&self) -> Result<InEarState, EarError> {
        self.require_support("in-ear detection", |base| base.supports_in_ear_detection())
            .await?;
        let conn = self.inner.connection.lock().await;
        conn.transact(
            command::REQUEST_IN_EAR_STATUS,
            &[],
            |packet| {
                if packet.command == response::IN_EAR {
                    packet.payload.get(2).map(|&value| InEarState {
                        detection_enabled: value == 1,
                    })
                } else {
                    None
                }
            },
            "in_ear",
        )
        .await
    }

    pub async fn set_in_ear_detection(&self, enabled: bool) -> Result<(), EarError> {
        self.require_support("in-ear detection", |base| base.supports_in_ear_detection())
            .await?;
        let conn = self.inner.connection.lock().await;
        let payload = [0x01, 0x01, if enabled { 0x01 } else { 0x00 }];
        conn.send_command(command::CMD_SET_IN_EAR, &payload).await?;
        Ok(())
    }

    pub async fn read_latency(&self) -> Result<LatencyState, EarError> {
        let conn = self.inner.connection.lock().await;
        conn.transact(
            command::REQUEST_LATENCY_STATUS,
            &[],
            |packet| {
                if packet.command == response::LATENCY {
                    packet.payload.get(0).map(|&value| LatencyState {
                        low_latency_enabled: value == 1,
                    })
                } else {
                    None
                }
            },
            "latency",
        )
        .await
    }

    pub async fn set_latency(&self, enabled: bool) -> Result<(), EarError> {
        let conn = self.inner.connection.lock().await;
        let payload = if enabled { [0x01, 0x00] } else { [0x02, 0x00] };
        conn.send_command(command::CMD_SET_LATENCY, &payload)
            .await?;
        Ok(())
    }

    pub async fn read_firmware(&self) -> Result<FirmwareInfo, EarError> {
        let conn = self.inner.connection.lock().await;
        conn.transact(
            command::REQUEST_FIRMWARE,
            &[],
            |packet| {
                if packet.command == response::FIRMWARE {
                    Some(FirmwareInfo {
                        version: String::from_utf8_lossy(&packet.payload).trim().to_string(),
                    })
                } else {
                    None
                }
            },
            "firmware",
        )
        .await
    }

    pub async fn launch_ear_fit_test(&self) -> Result<(), EarError> {
        let conn = self.inner.connection.lock().await;
        conn.send_command(command::CMD_START_EAR_FIT_TEST, &[0x01])
            .await?;
        Ok(())
    }

    pub async fn read_ear_fit_result(&self) -> Result<EarFitResult, EarError> {
        let conn = self.inner.connection.lock().await;
        conn.transact(
            command::CMD_START_EAR_FIT_TEST,
            &[0x00],
            |packet| {
                if packet.command == response::EAR_FIT_RESULT {
                    let left = packet.payload.get(0).copied().unwrap_or_default();
                    let right = packet.payload.get(1).copied().unwrap_or_default();
                    Some(EarFitResult { left, right })
                } else {
                    None
                }
            },
            "ear_fit_result",
        )
        .await
    }

    pub async fn read_gestures(&self) -> Result<Vec<GestureSlot>, EarError> {
        let conn = self.inner.connection.lock().await;
        conn.transact(
            command::REQUEST_GESTURES,
            &[],
            |packet| {
                if packet.command == response::GESTURES {
                    Some(parse_gestures(&packet.payload))
                } else {
                    None
                }
            },
            "gestures",
        )
        .await
    }

    pub async fn set_gesture(&self, slot: &GestureSlot) -> Result<(), EarError> {
        let conn = self.inner.connection.lock().await;
        let payload = [
            0x01,
            slot.device,
            slot.common,
            slot.gesture_type,
            slot.action,
        ];
        conn.send_command(command::CMD_SET_GESTURE, &payload)
            .await?;
        Ok(())
    }

    pub async fn read_led_case_colors(&self) -> Result<LedColorSet, EarError> {
        self.require_support("case led color", |base| base.supports_case_led())
            .await?;
        let conn = self.inner.connection.lock().await;
        conn.transact(
            command::REQUEST_LED_CASE_COLORS,
            &[],
            |packet| {
                if packet.command == response::LED_CASE_COLORS {
                    Some(parse_led_colors(&packet.payload))
                } else {
                    None
                }
            },
            "case_led_colors",
        )
        .await
    }

    pub async fn set_led_case_colors(&self, colors: &LedColorSet) -> Result<(), EarError> {
        self.require_support("case led color", |base| base.supports_case_led())
            .await?;
        let conn = self.inner.connection.lock().await;
        let mut payload = Vec::with_capacity(1 + colors.pixels.len() * 4);
        payload.push(colors.pixels.len() as u8);
        for (index, LedColor(rgb)) in colors.pixels.iter().cloned().enumerate() {
            payload.push((index + 1) as u8);
            payload.extend_from_slice(&rgb);
        }
        conn.send_command(command::CMD_SET_LED_CASE_COLORS, &payload)
            .await?;
        Ok(())
    }

    pub async fn ring_buds(&self, enable: bool, side: Option<EarSide>) -> Result<(), EarError> {
        let base = self.model_base().await;
        let conn = self.inner.connection.lock().await;
        let payload = if base == ModelBase::B181 {
            if enable { vec![0x01] } else { vec![0x00] }
        } else {
            let device = match side {
                Some(EarSide::Left) => 0x02,
                _ => 0x03,
            };
            vec![device, if enable { 0x01 } else { 0x00 }]
        };
        conn.send_command(command::CMD_RING, &payload).await?;
        Ok(())
    }

    async fn model_base(&self) -> ModelBase {
        self.inner
            .model
            .read()
            .await
            .as_ref()
            .map(|m| m.base)
            .unwrap_or(ModelBase::Unknown)
    }

    async fn require_support<F>(&self, label: &'static str, predicate: F) -> Result<(), EarError>
    where
        F: Fn(ModelBase) -> bool,
    {
        let base = self.model_base().await;
        if predicate(base) {
            Ok(())
        } else {
            Err(EarError::Unsupported(label))
        }
    }
}

fn parse_serial_number(payload: &[u8]) -> Option<String> {
    if payload.len() < 8 {
        return None;
    }
    let text = String::from_utf8_lossy(&payload[7..]);
    for line in text.lines() {
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() == 3 {
            if parts[1].trim() == "4" {
                let value = parts[2].trim();
                if !value.is_empty() {
                    return Some(value.to_string());
                }
            }
        }
    }
    None
}

fn derive_sku_from_serial(serial: &str) -> Option<String> {
    if serial == "12345678901234567" {
        return Some("01".to_string());
    }
    if serial.len() < 6 {
        return None;
    }
    let head = &serial[..2];
    if head == "MA" {
        let year = &serial[6..8];
        if year == "22" || year == "23" {
            return Some("14".to_string());
        } else if year == "24" {
            return Some("11200005".to_string());
        }
    } else if head == "SH" || head == "13" {
        return serial.get(4..6).map(|value| value.to_string());
    }
    None
}

fn parse_battery_payload(payload: &[u8]) -> BatteryStatus {
    let mut status = BatteryStatus::empty();
    if payload.is_empty() {
        return status;
    }
    let count = payload[0] as usize;
    for i in 0..count {
        let idx = 1 + i * 2;
        if idx + 1 >= payload.len() {
            break;
        }
        let device_id = payload[idx];
        let level_byte = payload[idx + 1];
        let level = level_byte & 0x7F;
        let charging = (level_byte & 0x80) == 0x80;
        let reading = BatteryReading::Level {
            percent: level,
            charging,
        };
        match device_id {
            0x02 => status.left = reading,
            0x03 => status.right = reading,
            0x04 => status.case = reading,
            _ => {}
        }
    }
    status
}

fn decode_custom_eq(payload: &[u8]) -> Option<CustomEq> {
    if payload.len() < 45 {
        return None;
    }
    let mut levels = Vec::new();
    for band in 0..3 {
        let offset = 6 + band * 13;
        if offset + 4 > payload.len() {
            return None;
        }
        let slice = &payload[offset..offset + 4];
        levels.push(decode_eq_float(slice));
    }
    if levels.len() == 3 {
        Some(CustomEq {
            bass: levels[2],
            mid: levels[0],
            treble: levels[1],
        })
    } else {
        None
    }
}

fn encode_custom_eq(eq: CustomEq) -> Vec<u8> {
    let mut payload = vec![
        0x03, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x75, 0x44, 0xc3,
        0xf5, 0x28, 0x3f, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0xc0, 0x5a, 0x45, 0x00, 0x00, 0x80,
        0x3f, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0c, 0x43, 0xcd, 0xcc, 0x4c, 0x3f, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ];
    let values = [eq.mid, eq.treble, eq.bass];
    let highest = values.iter().fold(0.0_f32, |acc, &v| acc.max(v)).abs();
    let total_bytes = encode_eq_float(-highest, true);
    payload[1..5].copy_from_slice(&total_bytes);
    for (index, value) in values.iter().enumerate() {
        let bytes = encode_eq_float(*value, false);
        let offset = 6 + index * 13;
        payload[offset..offset + 4].copy_from_slice(&bytes);
    }
    payload
}

fn encode_eq_float(value: f32, total: bool) -> [u8; 4] {
    if total && value >= 0.0 {
        return [0x00, 0x00, 0x00, 0x80];
    }
    let mut bytes = value.to_bits().to_be_bytes();
    if value != 0.0 && bytes[0] == 0 && bytes[1] == 0 && bytes[2] == 0 {
        bytes[3] |= 0x80;
    }
    bytes.swap(0, 3);
    bytes.swap(1, 2);
    bytes
}

fn decode_eq_float(bytes: &[u8]) -> f32 {
    if bytes.len() < 4 {
        return 0.0;
    }
    let mut slice = [bytes[3], bytes[2], bytes[1], bytes[0]];
    if slice[0] == 0 && slice[1] == 0 && slice[2] == 0 && (slice[3] & 0x80) == 0x80 {
        slice[3] &= 0x7F;
        -f32::from_bits(u32::from_be_bytes(slice))
    } else {
        f32::from_bits(u32::from_be_bytes(slice))
    }
}

fn parse_gestures(payload: &[u8]) -> Vec<GestureSlot> {
    if payload.is_empty() {
        return Vec::new();
    }
    let count = payload[0] as usize;
    let mut gestures = Vec::with_capacity(count);
    for i in 0..count {
        let base = 1 + i * 4;
        if base + 3 >= payload.len() {
            break;
        }
        gestures.push(GestureSlot {
            device: payload[base],
            common: payload[base + 1],
            gesture_type: payload[base + 2],
            action: payload[base + 3],
        });
    }
    gestures
}

fn parse_led_colors(payload: &[u8]) -> LedColorSet {
    if payload.is_empty() {
        return LedColorSet { pixels: Vec::new() };
    }
    let count = payload[0] as usize;
    let mut colors = Vec::with_capacity(count);
    for index in 0..count {
        let base = 2 + index * 4;
        if base + 2 >= payload.len() {
            break;
        }
        colors.push(LedColor([
            payload[base],
            payload[base + 1],
            payload[base + 2],
        ]));
    }
    LedColorSet { pixels: colors }
}
