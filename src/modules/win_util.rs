//! Win32 Registry and Hardware API wrappers for Windows target.

#[cfg(windows)]
pub mod ffi {
    pub const HKEY_LOCAL_MACHINE: isize = 0x80000002_u32 as i32 as isize;
    pub const HKEY_CURRENT_USER: isize = 0x80000001_u32 as i32 as isize;

    pub const KEY_READ: u32 = 0x20019;
    pub const REG_SZ: u32 = 1;
    pub const REG_EXPAND_SZ: u32 = 2;
    pub const REG_DWORD: u32 = 4;
    pub const REG_QWORD: u32 = 11;

    #[repr(C)]
    #[allow(non_snake_case)]
    pub struct MEMORYSTATUSEX {
        pub dwLength: u32,
        pub dwMemoryLoad: u32,
        pub ullTotalPhys: u64,
        pub ullAvailPhys: u64,
        pub ullTotalPageFile: u64,
        pub ullAvailPageFile: u64,
        pub ullTotalVirtual: u64,
        pub ullAvailVirtual: u64,
        pub ullAvailExtendedVirtual: u64,
    }

    #[repr(C)]
    #[allow(non_snake_case)]
    pub struct SYSTEM_POWER_STATUS {
        pub ACLineStatus: u8,
        pub BatteryFlag: u8,
        pub BatteryLifePercent: u8,
        pub SystemStatusFlag: u8,
        pub BatteryLifeTime: u32,
        pub BatteryFullLifeTime: u32,
    }

    #[repr(C)]
    #[allow(non_snake_case)]
    pub struct SYSTEMTIME {
        pub wYear: u16,
        pub wMonth: u16,
        pub wDayOfWeek: u16,
        pub wDay: u16,
        pub wHour: u16,
        pub wMinute: u16,
        pub wSecond: u16,
        pub wMilliseconds: u16,
    }

    #[repr(C)]
    #[allow(non_snake_case)]
    pub struct TIME_ZONE_INFORMATION {
        pub Bias: i32,
        pub StandardName: [u16; 32],
        pub StandardDate: SYSTEMTIME,
        pub StandardBias: i32,
        pub DaylightName: [u16; 32],
        pub DaylightDate: SYSTEMTIME,
        pub DaylightBias: i32,
    }

    #[allow(non_snake_case)]
    #[link(name = "advapi32")]
    extern "system" {
        pub fn RegOpenKeyExW(
            hKey: isize,
            lpSubKey: *const u16,
            ulOptions: u32,
            samDesired: u32,
            phkResult: *mut isize,
        ) -> i32;

        pub fn RegQueryValueExW(
            hKey: isize,
            lpValueName: *const u16,
            lpReserved: *const u32,
            lpType: *mut u32,
            lpData: *mut u8,
            lpcbData: *mut u32,
        ) -> i32;

        pub fn RegEnumKeyExW(
            hKey: isize,
            dwIndex: u32,
            lpName: *mut u16,
            lpcchName: *mut u32,
            lpReserved: *const u32,
            lpClass: *mut u16,
            lpcchClass: *mut u32,
            lpftLastWriteTime: *mut u64,
        ) -> i32;

        pub fn RegCloseKey(hKey: isize) -> i32;
    }

    #[repr(C)]
    #[allow(non_snake_case)]
    pub struct PROCESSENTRY32W {
        pub dwSize: u32,
        pub cntUsage: u32,
        pub th32ProcessID: u32,
        pub th32DefaultHeapID: usize,
        pub th32ModuleID: u32,
        pub cntThreads: u32,
        pub th32ParentProcessID: u32,
        pub pcPriClassBase: i32,
        pub dwFlags: u32,
        pub szExeFile: [u16; 260],
    }

    #[link(name = "kernel32")]
    extern "system" {
        pub fn GetTickCount64() -> u64;
        pub fn GlobalMemoryStatusEx(lpBuffer: *mut MEMORYSTATUSEX) -> i32;
        pub fn GetDiskFreeSpaceExW(
            lpDirectoryName: *const u16,
            lpFreeBytesAvailableToCaller: *mut u64,
            lpTotalNumberOfBytes: *mut u64,
            lpTotalNumberOfFreeBytes: *mut u64,
        ) -> i32;
        pub fn GetLogicalDrives() -> u32;
        pub fn GetDriveTypeW(lpRootPathName: *const u16) -> u32;
        pub fn GetSystemPowerStatus(lpSystemPowerStatus: *mut SYSTEM_POWER_STATUS) -> i32;
        pub fn GetTimeZoneInformation(lpTimeZoneInformation: *mut TIME_ZONE_INFORMATION) -> u32;
        pub fn GetVolumeInformationW(
            lpRootPathName: *const u16,
            lpVolumeNameBuffer: *mut u16,
            nVolumeNameSize: u32,
            lpVolumeSerialNumber: *mut u32,
            lpMaximumComponentLength: *mut u32,
            lpFileSystemFlags: *mut u32,
            lpFileSystemNameBuffer: *mut u16,
            nFileSystemNameSize: u32,
        ) -> i32;
        pub fn CreateToolhelp32Snapshot(dwFlags: u32, th32ProcessID: u32) -> isize;
        pub fn Process32FirstW(hSnapshot: isize, lppe: *mut PROCESSENTRY32W) -> i32;
        pub fn Process32NextW(hSnapshot: isize, lppe: *mut PROCESSENTRY32W) -> i32;
        pub fn GetCurrentProcessId() -> u32;
        pub fn CloseHandle(hObject: isize) -> i32;
    }

    fn to_wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(std::iter::once(0)).collect()
    }

    pub fn reg_open_key(root: isize, subkey: &str) -> Option<isize> {
        let subkey_w = to_wide(subkey);
        let mut hkey: isize = 0;
        let res = unsafe { RegOpenKeyExW(root, subkey_w.as_ptr(), 0, KEY_READ, &mut hkey) };
        if res == 0 && hkey != 0 {
            Some(hkey)
        } else {
            None
        }
    }

    pub fn reg_close_key(hkey: isize) {
        unsafe {
            RegCloseKey(hkey);
        }
    }

    pub fn reg_query_string(hkey: isize, value_name: &str) -> Option<String> {
        let val_w = to_wide(value_name);
        let mut val_type: u32 = 0;
        let mut data_size: u32 = 0;

        let res = unsafe {
            RegQueryValueExW(
                hkey,
                val_w.as_ptr(),
                std::ptr::null(),
                &mut val_type,
                std::ptr::null_mut(),
                &mut data_size,
            )
        };
        if res != 0 || data_size == 0 {
            return None;
        }

        let mut buffer: Vec<u8> = vec![0; data_size as usize];
        let res = unsafe {
            RegQueryValueExW(
                hkey,
                val_w.as_ptr(),
                std::ptr::null(),
                &mut val_type,
                buffer.as_mut_ptr(),
                &mut data_size,
            )
        };
        if res != 0 {
            return None;
        }

        if val_type == REG_SZ || val_type == REG_EXPAND_SZ {
            let u16_slice = unsafe {
                std::slice::from_raw_parts(buffer.as_ptr() as *const u16, (data_size / 2) as usize)
            };
            let s = String::from_utf16_lossy(u16_slice);
            Some(s.trim_matches('\0').trim().to_string())
        } else if val_type == REG_DWORD && data_size >= 4 {
            let val = u32::from_ne_bytes(buffer[0..4].try_into().ok()?);
            Some(val.to_string())
        } else {
            None
        }
    }

    pub fn reg_query_u32(hkey: isize, value_name: &str) -> Option<u32> {
        let val_w = to_wide(value_name);
        let mut val_type: u32 = 0;
        let mut data_size: u32 = 4;
        let mut data: u32 = 0;

        let res = unsafe {
            RegQueryValueExW(
                hkey,
                val_w.as_ptr(),
                std::ptr::null(),
                &mut val_type,
                &mut data as *mut u32 as *mut u8,
                &mut data_size,
            )
        };
        if res == 0 {
            if val_type == REG_DWORD {
                Some(data)
            } else if val_type == REG_SZ {
                let s = reg_query_string(hkey, value_name)?;
                s.parse::<u32>().ok()
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn reg_query_u64(hkey: isize, value_name: &str) -> Option<u64> {
        let val_w = to_wide(value_name);
        let mut val_type: u32 = 0;
        let mut data_size: u32 = 8;
        let mut data: u64 = 0;

        let res = unsafe {
            RegQueryValueExW(
                hkey,
                val_w.as_ptr(),
                std::ptr::null(),
                &mut val_type,
                &mut data as *mut u64 as *mut u8,
                &mut data_size,
            )
        };
        if res == 0 {
            if val_type == REG_QWORD {
                Some(data)
            } else if val_type == REG_DWORD {
                Some(data as u32 as u64)
            } else if val_type == REG_SZ {
                let s = reg_query_string(hkey, value_name)?;
                s.parse::<u64>().ok()
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn reg_read_string(root: isize, subkey: &str, value_name: &str) -> Option<String> {
        let hkey = reg_open_key(root, subkey)?;
        let val = reg_query_string(hkey, value_name);
        reg_close_key(hkey);
        val
    }

    pub fn reg_read_u32(root: isize, subkey: &str, value_name: &str) -> Option<u32> {
        let hkey = reg_open_key(root, subkey)?;
        let val = reg_query_u32(hkey, value_name);
        reg_close_key(hkey);
        val
    }

    pub fn reg_read_u64(root: isize, subkey: &str, value_name: &str) -> Option<u64> {
        let hkey = reg_open_key(root, subkey)?;
        let val = reg_query_u64(hkey, value_name);
        reg_close_key(hkey);
        val
    }

    pub fn reg_enum_subkeys(root: isize, subkey: &str) -> Vec<String> {
        let Some(hkey) = reg_open_key(root, subkey) else {
            return Vec::new();
        };

        let mut subkeys = Vec::new();
        let mut index = 0;
        let mut name_buf = vec![0u16; 256];

        loop {
            let mut name_len = name_buf.len() as u32;
            let res = unsafe {
                RegEnumKeyExW(
                    hkey,
                    index,
                    name_buf.as_mut_ptr(),
                    &mut name_len,
                    std::ptr::null(),
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                )
            };
            if res != 0 {
                break;
            }
            let s = String::from_utf16_lossy(&name_buf[..name_len as usize]);
            let clean = s.trim().to_string();
            if !clean.is_empty() {
                subkeys.push(clean);
            }
            index += 1;
        }

        reg_close_key(hkey);
        subkeys
    }

    pub fn get_parent_process_chain(max_depth: usize) -> Vec<(u32, String)> {
        let mut chain = Vec::new();
        let h_snap = unsafe { CreateToolhelp32Snapshot(0x00000002, 0) };
        if h_snap == 0 || h_snap == -1 {
            return chain;
        }

        let mut processes = std::collections::HashMap::new();
        let mut entry = unsafe { std::mem::zeroed::<PROCESSENTRY32W>() };
        entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;

        if unsafe { Process32FirstW(h_snap, &mut entry) } != 0 {
            loop {
                let name_len = entry
                    .szExeFile
                    .iter()
                    .position(|&c| c == 0)
                    .unwrap_or(entry.szExeFile.len());
                let exe_name = String::from_utf16_lossy(&entry.szExeFile[..name_len]);
                processes.insert(entry.th32ProcessID, (entry.th32ParentProcessID, exe_name));

                if unsafe { Process32NextW(h_snap, &mut entry) } == 0 {
                    break;
                }
            }
        }
        unsafe { CloseHandle(h_snap) };

        let mut curr_pid = unsafe { GetCurrentProcessId() };
        for _ in 0..max_depth {
            if let Some(&(ppid, ref name)) = processes.get(&curr_pid) {
                if ppid == 0 || ppid == curr_pid {
                    break;
                }
                chain.push((ppid, name.clone()));
                curr_pid = ppid;
            } else {
                break;
            }
        }
        chain
    }
}
