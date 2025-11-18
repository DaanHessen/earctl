use std::path::PathBuf;

use tokio::process::Command;

use crate::error::EarError;

const NOTHING_SPP_UUID: &str = "aeac4a03-dff5-498f-843a-34487cf133eb";

#[derive(Debug, Clone)]
pub struct BluetoothDevice {
    pub address: String,
    pub name: String,
}

pub async fn resolve_connected_device(
    preferred_address: Option<String>,
    preferred_name: Option<String>,
) -> Result<BluetoothDevice, EarError> {
    if let Some(address) = preferred_address {
        verify_device_connected(&address).await?;
        let name = device_name(&address)
            .await
            .unwrap_or_else(|| "".to_string());
        return Ok(BluetoothDevice { address, name });
    }

    let connected = list_connected_devices().await?;
    if connected.is_empty() {
        return Err(EarError::Detection(
            "no connected Bluetooth devices were found; please connect your earbuds first"
                .to_string(),
        ));
    }
    if let Some(name) = preferred_name {
        if let Some(found) = connected
            .into_iter()
            .find(|device| device.name.to_lowercase().contains(&name.to_lowercase()))
        {
            verify_device_connected(&found.address).await?;
            return Ok(found);
        }
        return Err(EarError::Detection(format!(
            "could not find connected device matching name '{}'",
            name
        )));
    }
    let first = connected.into_iter().next().unwrap();
    verify_device_connected(&first.address).await?;
    Ok(first)
}

pub fn rfcomm_path(name: &str) -> PathBuf {
    if name.starts_with("/dev/") {
        PathBuf::from(name)
    } else {
        PathBuf::from(format!("/dev/{}", name))
    }
}

pub fn next_available_rfcomm_name() -> String {
    for index in 0..10 {
        let name = format!("rfcomm{}", index);
        if !rfcomm_path(&name).exists() {
            return name;
        }
    }
    "rfcomm0".to_string()
}

pub async fn list_connected_devices() -> Result<Vec<BluetoothDevice>, EarError> {
    let output = run_command("bluetoothctl", &["devices", "Connected"]).await?;
    let devices = output
        .lines()
        .filter_map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 3 {
                return None;
            }
            let address = parts[1].to_string();
            let name = parts[2..].join(" ");
            Some(BluetoothDevice { address, name })
        })
        .collect();
    Ok(devices)
}

async fn verify_device_connected(address: &str) -> Result<(), EarError> {
    let output = run_command("bluetoothctl", &["info", address]).await?;
    if !output.to_lowercase().contains("connected: yes") {
        return Err(EarError::Detection(format!(
            "bluetooth device {} is not currently connected",
            address
        )));
    }
    Ok(())
}

async fn device_name(address: &str) -> Option<String> {
    run_command("bluetoothctl", &["info", address])
        .await
        .ok()
        .and_then(|info| {
            info.lines()
                .find(|line| line.trim_start().starts_with("Name:"))
                .map(|line| {
                    line.split_once(':')
                        .map(|(_, value)| value.trim().to_string())
                })
                .flatten()
        })
}

async fn run_command(cmd: &str, args: &[&str]) -> Result<String, EarError> {
    let output = Command::new(cmd)
        .args(args)
        .output()
        .await
        .map_err(|err| EarError::Detection(format!("failed to run `{}`: {}", cmd, err)))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(EarError::CommandFailed {
            command: format!("{} {}", cmd, args.join(" ")),
            output: stderr,
        });
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

pub async fn detect_rfcomm_channel(address: &str) -> Result<u8, EarError> {
    let output = run_command("sdptool", &["search", "--bdaddr", address, "SP"]).await?;
    let mut tracking_target = false;
    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("Service Name:") {
            // Reset for each record
            tracking_target = trimmed.to_lowercase().contains("nt link");
            continue;
        }
        if trimmed.starts_with("UUID 128:") {
            if trimmed
                .to_lowercase()
                .contains(&NOTHING_SPP_UUID.to_lowercase())
            {
                tracking_target = true;
            }
            continue;
        }
        if trimmed.starts_with("Channel:") && tracking_target {
            if let Ok(channel) = trimmed.trim_start_matches("Channel:").trim().parse::<u8>() {
                return Ok(channel);
            }
        }
    }
    Err(EarError::Detection(
        "failed to detect RFCOMM channel; provide `channel` manually or keep Nothing X open once to expose the NT LINK service"
            .into(),
    ))
}
