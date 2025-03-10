use std::collections::HashMap;
use sysinfo::{Disks, Networks, System};
use log::trace;

#[derive(Debug)]
pub struct CpuInfo {
    pub id: String,
    pub cpus: usize,
}

#[derive(Debug)]
pub struct HardDiskInfo {
    pub path: String,
    pub usage: i32, /*percent*/
}

#[derive(Debug)]
pub struct NetworkInfo {
    pub name: String,
    pub mac: String,
}

#[derive(Debug)]
pub struct SystemInfo {
    pub server_id: String,
    pub server_sys_name: String,
    pub server_kernel_version: String,
    pub server_os_version: String,
    pub server_host_name: String,
    pub cpu_info: CpuInfo,
    pub networks: Vec<NetworkInfo>,
    pub hard_disks: Vec<HardDiskInfo>,
}

impl SystemInfo {
    pub fn new() -> Self {
        let mut info = Self::default();

        let mut sys = System::new_all();
        sys.refresh_all();

        //
        info.server_sys_name = System::name().unwrap_or("".to_string());
        info.server_kernel_version = System::kernel_version().unwrap_or("".to_string());
        info.server_os_version = System::os_version().unwrap_or("".to_string());
        info.server_host_name = System::host_name().unwrap_or("".to_string());

        //
        info.cpu_info = CpuInfo {
            id: "".to_string(),
            cpus: sys.cpus().len(),
        };

        //
        let disks = Disks::new_with_refreshed_list();
        for disk in &disks {
            info.hard_disks.push(HardDiskInfo {
                path: disk.mount_point().to_str().unwrap_or("").to_string(),
                usage: 0,
            });
        }

        //
        let networks = Networks::new_with_refreshed_list();
        for (interface_name, data) in &networks {
            info.networks.push(NetworkInfo {
                name: interface_name.to_string(),
                mac: data.mac_address().to_string(),
            });
        }

        let mut unique_info = String::from("");
        unique_info += info.server_host_name.as_str();
        unique_info += info.cpu_info.cpus.to_string().as_str();
        info.networks.iter().for_each(|n| {
            unique_info += n.mac.as_str();
        });
        info.server_id = crate::md5_hex(&unique_info);

        info
    }
}

impl Default for SystemInfo {
    fn default() -> Self {
        SystemInfo {
            server_id: "".to_string(),
            server_sys_name: "".to_string(),
            server_kernel_version: "".to_string(),
            server_os_version: "".to_string(),
            server_host_name: "".to_string(),
            cpu_info: CpuInfo { id: "".to_string(), cpus: 0 },
            networks: vec![],
            hard_disks: vec![],
        }
    }
}