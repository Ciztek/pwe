use cpal::traits::{DeviceTrait, HostTrait};
use tracing::{error, info};

#[derive(Debug, Clone)]
pub struct AudioDevice {
    pub name: String,
    pub is_default: bool,
}

pub fn list_output_devices() -> Vec<AudioDevice> {
    let host = cpal::default_host();
    let mut devices = Vec::new();

    // Get default device
    let default_device_name = host.default_output_device().and_then(|d| d.name().ok());

    // List all output devices
    match host.output_devices() {
        Ok(device_iter) => {
            for device in device_iter {
                if let Ok(name) = device.name() {
                    let is_default = Some(&name) == default_device_name.as_ref();
                    devices.push(AudioDevice { name, is_default });
                }
            }
        },
        Err(e) => {
            error!("Failed to enumerate output devices: {}", e);
        },
    }

    if devices.is_empty() {
        info!("No output devices found, adding default");
        devices.push(AudioDevice {
            name: "Default Output".to_string(),
            is_default: true,
        });
    }

    devices
}

#[allow(dead_code)]
pub fn list_input_devices() -> Vec<AudioDevice> {
    let host = cpal::default_host();
    let mut devices = Vec::new();

    // Get default device
    let default_device_name = host.default_input_device().and_then(|d| d.name().ok());

    // List all input devices
    match host.input_devices() {
        Ok(device_iter) => {
            for device in device_iter {
                if let Ok(name) = device.name() {
                    let is_default = Some(&name) == default_device_name.as_ref();
                    devices.push(AudioDevice { name, is_default });
                }
            }
        },
        Err(e) => {
            error!("Failed to enumerate input devices: {}", e);
        },
    }

    if devices.is_empty() {
        info!("No input devices found, adding default");
        devices.push(AudioDevice {
            name: "Default Input".to_string(),
            is_default: true,
        });
    }

    devices
}
