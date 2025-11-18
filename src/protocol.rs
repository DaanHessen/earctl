use crate::error::EarError;

pub const HEADER_MAGIC: [u8; 3] = [0x55, 0x60, 0x01];
const HEADER_LEN: usize = 8;
const CRC_LEN: usize = 2;

#[derive(Debug, Clone)]
pub struct EarPacket {
    pub command: u16,
    pub operation_id: u8,
    pub payload: Vec<u8>,
}

pub mod command {
    pub const REQUEST_SERIAL: u16 = 0xC006;
    pub const REQUEST_BATTERY: u16 = 0xC007;
    pub const REQUEST_LED_CASE_COLORS: u16 = 0xC017;
    pub const REQUEST_GESTURES: u16 = 0xC018;
    pub const REQUEST_ANC: u16 = 0xC01E;
    pub const REQUEST_EQ: u16 = 0xC01F;
    pub const REQUEST_PERSONALIZED_ANC: u16 = 0xC020;
    pub const REQUEST_IN_EAR_STATUS: u16 = 0xC00E;
    pub const REQUEST_LATENCY_STATUS: u16 = 0xC041;
    pub const REQUEST_FIRMWARE: u16 = 0xC042;
    pub const REQUEST_CUSTOM_EQ: u16 = 0xC044;
    pub const REQUEST_ADVANCED_EQ: u16 = 0xC04C;
    pub const REQUEST_ENHANCED_BASS: u16 = 0xC04E;
    pub const REQUEST_LISTENING_MODE: u16 = 0xC050;

    pub const CMD_RING: u16 = 0xF002;
    pub const CMD_SET_GESTURE: u16 = 0xF003;
    pub const CMD_SET_IN_EAR: u16 = 0xF004;
    pub const CMD_SET_LED_CASE_COLORS: u16 = 0xF00D;
    pub const CMD_SET_ANC: u16 = 0xF00F;
    pub const CMD_SET_EQ: u16 = 0xF010;
    pub const CMD_SET_PERSONALIZED_ANC: u16 = 0xF011;
    pub const CMD_START_EAR_FIT_TEST: u16 = 0xF014;
    pub const CMD_SET_LISTENING_MODE: u16 = 0xF01D;
    pub const CMD_SET_LATENCY: u16 = 0xF040;
    pub const CMD_SET_CUSTOM_EQ: u16 = 0xF041;
    pub const CMD_SET_ADVANCED_EQ_ENABLED: u16 = 0xF04F;
    pub const CMD_SET_ENHANCED_BASS: u16 = 0xF051;
}

pub mod response {
    pub const SERIAL: u16 = 0x4006;
    pub const BATTERY_PRIMARY: u16 = 0xE001;
    pub const BATTERY_SECONDARY: u16 = 0x4007;
    pub const ANC_PRIMARY: u16 = 0xE003;
    pub const ANC_SECONDARY: u16 = 0x401E;
    pub const EQ_PRIMARY: u16 = 0x401F;
    pub const EQ_LISTENING_MODE: u16 = 0x4050;
    pub const FIRMWARE: u16 = 0x4042;
    pub const CUSTOM_EQ: u16 = 0x4044;
    pub const ADVANCED_EQ: u16 = 0x404C;
    pub const ENHANCED_BASS: u16 = 0x404E;
    pub const LED_CASE_COLORS: u16 = 0x4017;
    pub const GESTURES: u16 = 0x4018;
    pub const PERSONALIZED_ANC: u16 = 0x4020;
    pub const IN_EAR: u16 = 0x400E;
    pub const LATENCY: u16 = 0x4041;
    pub const EAR_FIT_RESULT: u16 = 0xE00D;
}

impl EarPacket {
    pub fn encode(command: u16, operation_id: u8, payload: &[u8]) -> Vec<u8> {
        let mut packet = Vec::with_capacity(HEADER_LEN + payload.len() + CRC_LEN);
        packet.extend_from_slice(&HEADER_MAGIC);
        packet.extend_from_slice(&command.to_le_bytes());
        packet.push(payload.len() as u8);
        packet.push(0x00);
        packet.push(operation_id);
        packet.extend_from_slice(payload);
        let crc = crc16(&packet);
        packet.extend_from_slice(&crc.to_le_bytes());
        packet
    }

    pub fn try_parse(buffer: &mut Vec<u8>) -> Result<Option<EarPacket>, EarError> {
        loop {
            if buffer.len() < HEADER_LEN {
                return Ok(None);
            }
            let Some(start_index) = buffer.iter().position(|&byte| byte == HEADER_MAGIC[0]) else {
                buffer.clear();
                return Ok(None);
            };
            if start_index > 0 {
                buffer.drain(0..start_index);
            }
            if buffer.len() < HEADER_LEN {
                return Ok(None);
            }
            if buffer[1] != HEADER_MAGIC[1] || buffer[2] != HEADER_MAGIC[2] {
                buffer.drain(0..1);
                continue;
            }
            let payload_len = buffer[5] as usize;
            let total_len = HEADER_LEN + payload_len + CRC_LEN;
            if buffer.len() < total_len {
                return Ok(None);
            }
            let packet_bytes: Vec<u8> = buffer.drain(0..total_len).collect();
            let crc_expected =
                u16::from_le_bytes([packet_bytes[total_len - 2], packet_bytes[total_len - 1]]);
            let crc_actual = crc16(&packet_bytes[..total_len - CRC_LEN]);
            if crc_actual != crc_expected {
                return Err(EarError::CrcMismatch);
            }

            let command = u16::from_le_bytes([packet_bytes[3], packet_bytes[4]]);
            let operation_id = packet_bytes[7];
            let payload = packet_bytes[HEADER_LEN..HEADER_LEN + payload_len].to_vec();

            return Ok(Some(EarPacket {
                command,
                operation_id,
                payload,
            }));
        }
    }
}

pub fn crc16(buffer: &[u8]) -> u16 {
    let mut crc: u16 = 0xFFFF;
    for &byte in buffer {
        crc ^= byte as u16;
        for _ in 0..8 {
            if crc & 1 == 1 {
                crc = (crc >> 1) ^ 0xA001;
            } else {
                crc >>= 1;
            }
        }
    }
    crc
}
