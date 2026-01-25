mod controller;

pub use controller::*;

use network::{IpAddress, IpCidr, Route};
use smoltcp::phy::{Medium, TunTapInterface};
use std::process::Command;

pub const IP_ADDRESSES: &[IpCidr] = &[
    IpCidr::new_ipv4([192, 168, 69, 1], 24),
    IpCidr::new_ipv4([172, 0, 0, 1], 8),
    IpCidr::new_ipv6([0xfdaa, 0, 0, 0, 0, 0, 0, 1], 64),
    IpCidr::new_ipv6([0xfe80, 0, 0, 0, 0, 0, 0, 1], 64),
];

pub const ROUTES: &[Route] = &[
    Route::new_default_ipv4([192, 168, 69, 100]),
    Route::new_default_ipv6([0xfe80, 0, 0, 0, 0, 0, 0, 100]),
];

pub const DEFAULT_DNS_SERVERS: &[IpAddress] = &[
    IpAddress::new_ipv4([1, 1, 1, 1]),
    IpAddress::new_ipv4([1, 0, 0, 1]),
    IpAddress::new_ipv6([0x2606, 0x4700, 0x4700, 0, 0, 0, 0, 0x1111]),
    IpAddress::new_ipv6([0x2606, 0x4700, 0x4700, 0, 0, 0, 0, 0x1001]),
];

fn interface_exists(name: &str) -> bool {
    std::fs::metadata(format!("/sys/class/net/{}", name)).is_ok()
}

fn setup_tap_interface(name: &str) -> Result<(), String> {
    // Get the user from SUDO_USER environment variable
    log::information!(
        "Setting up TAP interface: {} (sudo permissions required)",
        name
    );

    let user = std::env::var("SUDO_USER")
        .unwrap_or_else(|_| std::env::var("USER").unwrap_or_else(|_| "root".to_string()));

    log::information!("Using user: {}", user);

    let commands: &[&[&str]] = &[
        &[
            "ip", "tuntap", "add", "name", name, "mode", "tap", "user", &user,
        ],
        &["ip", "link", "set", name, "up"],
        &["ip", "addr", "add", "192.168.69.100/24", "dev", name],
        &["ip", "-6", "addr", "add", "fe80::100/64", "dev", name],
        &["ip", "-6", "addr", "add", "fdaa::100/64", "dev", name],
        &[
            "iptables",
            "-t",
            "nat",
            "-I", // Change -A to -I
            "POSTROUTING",
            "1", // Insert at position 1
            "-s",
            "192.168.69.0/24",
            "-j",
            "MASQUERADE",
        ],
        &[
            "iptables",
            "-I", // Change -A to -I
            "FORWARD",
            "1", // Insert at position 1
            "-i",
            name,
            "-s",
            "192.168.69.0/24",
            "-j",
            "ACCEPT",
        ],
        &[
            "iptables",
            "-I", // Change -A to -I
            "FORWARD",
            "1", // Insert at position 1
            "-o",
            name,
            "-d",
            "192.168.69.0/24",
            "-j",
            "ACCEPT",
        ],
        &["sysctl", "-w", "net.ipv4.ip_forward=1"],
    ];

    // Commands that can fail if already exist (routes)
    let optional_commands: &[&[&str]] = &[
        &["ip", "-6", "route", "add", "fe80::/64", "dev", name],
        &["ip", "-6", "route", "add", "fdaa::/64", "dev", name],
    ];

    for &cmd_args in commands {
        let output = Command::new("sudo")
            .args(cmd_args)
            .output()
            .map_err(|e| format!("Failed to execute command: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!(
                "Command 'sudo {}' failed: {}",
                cmd_args.join(" "),
                stderr
            ));
        }
    }

    // Execute optional commands (ignore "File exists" errors)
    for &cmd_args in optional_commands {
        let output = Command::new("sudo")
            .args(cmd_args)
            .output()
            .map_err(|e| format!("Failed to execute command: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            // Ignore "File exists" errors for routes
            if !stderr.contains("File exists") {
                log::warning!("Command 'sudo {}' failed: {}", cmd_args.join(" "), stderr);
            }
        }
    }

    Ok(())
}

pub fn new(name: &str, tun: bool, tap: bool) -> Option<(TunTapInterface, TunTapControllerDevice)> {
    let medium = if tun {
        Medium::Ip
    } else if tap {
        Medium::Ethernet
    } else {
        log::error!("Either TUN or TAP mode must be specified.");
        return None;
    };

    if !interface_exists(name) {
        if let Err(e) = setup_tap_interface(name) {
            log::error!("Failed to setup TAP interface: {}", e);
            return None;
        }
    } else {
        log::information!("TAP interface {} already exists.", name);
    }

    let tuntap_device = TunTapInterface::new(name, medium)
        .map_err(|e| log::error!("Failed to create TUN/TAP device: {}", e))
        .ok()?;

    let controller = TunTapControllerDevice {};

    Some((tuntap_device, controller))
}
