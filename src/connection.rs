use std::time::Duration;

use bluer::rfcomm::{
    SocketAddr, Stream,
    stream::{OwnedReadHalf, OwnedWriteHalf},
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    sync::Mutex,
    time,
};

use crate::{error::EarError, protocol::EarPacket};

const READ_BUFFER_SIZE: usize = 512;
const DEFAULT_TIMEOUT_MS: u64 = 2000;

pub struct EarConnection {
    port_path: String,
    reader: Mutex<OwnedReadHalf>,
    writer: Mutex<OwnedWriteHalf>,
    read_buffer: Mutex<Vec<u8>>,
    operation_id: Mutex<u8>,
    timeout: Duration,
}

impl EarConnection {
    pub async fn open(address: bluer::Address, channel: u8) -> Result<Self, EarError> {
        let socket_addr = SocketAddr::new(address, channel);
        let port_path = socket_addr.to_string();

        tracing::info!("Connecting to RFCOMM {}", port_path);

        let stream = Stream::connect(socket_addr).await.map_err(|e| {
            EarError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("RFCOMM connect failed: {}", e),
            ))
        })?;

        let (reader, writer) = stream.into_split();

        Ok(Self {
            port_path,
            reader: Mutex::new(reader),
            writer: Mutex::new(writer),
            read_buffer: Mutex::new(Vec::with_capacity(READ_BUFFER_SIZE)),
            operation_id: Mutex::new(1),
            timeout: Duration::from_millis(DEFAULT_TIMEOUT_MS),
        })
    }

    pub fn port_path(&self) -> &str {
        &self.port_path
    }

    pub fn set_timeout(&mut self, timeout: Duration) {
        self.timeout = timeout;
    }

    async fn next_operation_id(&self) -> u8 {
        let mut op_id = self.operation_id.lock().await;
        *op_id = if *op_id >= 250 {
            1
        } else {
            op_id.wrapping_add(1).max(1)
        };
        *op_id
    }

    pub async fn send_command(&self, command: u16, payload: &[u8]) -> Result<u8, EarError> {
        let operation = self.next_operation_id().await;
        let packet = EarPacket::encode(command, operation, payload);

        let mut writer = self.writer.lock().await;
        writer.write_all(&packet).await.map_err(|e| {
            EarError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("RFCOMM write failed: {}", e),
            ))
        })?;
        writer.flush().await.map_err(|e| {
            EarError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("RFCOMM flush failed: {}", e),
            ))
        })?;

        tracing::debug!("sent command 0x{:04x} operation {}", command, operation);
        Ok(operation)
    }

    pub async fn transact<F, T>(
        &self,
        command: u16,
        payload: &[u8],
        mut matcher: F,
        label: &'static str,
    ) -> Result<T, EarError>
    where
        F: FnMut(&EarPacket) -> Option<T>,
    {
        self.send_command(command, payload).await?;
        let deadline = time::Instant::now() + self.timeout;
        loop {
            let packet = self.read_packet().await?;
            if let Some(value) = matcher(&packet) {
                return Ok(value);
            }
            if time::Instant::now() >= deadline {
                return Err(EarError::Timeout(label));
            }
        }
    }

    pub async fn read_packet(&self) -> Result<EarPacket, EarError> {
        let deadline = time::Instant::now() + self.timeout;
        let mut chunk = vec![0u8; READ_BUFFER_SIZE];

        loop {
            {
                let mut buffer = self.read_buffer.lock().await;
                if let Some(result) = EarPacket::try_parse(&mut buffer)? {
                    tracing::debug!("parsed packet: command=0x{:04x}", result.command);
                    return Ok(result);
                }
            }

            let remaining = deadline.saturating_duration_since(time::Instant::now());
            if remaining.is_zero() {
                return Err(EarError::Timeout("read packet"));
            }

            let mut reader = self.reader.lock().await;
            match time::timeout(remaining, reader.read(&mut chunk)).await {
                Ok(Ok(0)) => {
                    return Err(EarError::Io(std::io::Error::new(
                        std::io::ErrorKind::UnexpectedEof,
                        "RFCOMM stream closed",
                    )));
                }
                Ok(Ok(n)) => {
                    let mut buffer = self.read_buffer.lock().await;
                    buffer.extend_from_slice(&chunk[..n]);
                }
                Ok(Err(e)) => {
                    return Err(EarError::Io(e));
                }
                Err(_) => {
                    return Err(EarError::Timeout("read packet"));
                }
            }
        }
    }
}
