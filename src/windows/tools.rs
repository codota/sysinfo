//
// Sysinfo
//
// Copyright (c) 2018 Guillaume Gomez
//

use windows::processor::{self, Processor, Query};

use std::collections::HashMap;
use std::ffi::OsStr;
use std::mem::{size_of, zeroed};

use rayon::iter::{IntoParallelIterator, ParallelIterator};

use winapi::ctypes::c_void;

use winapi::shared::minwindef::{BYTE, DWORD, MAX_PATH, TRUE};
use winapi::um::fileapi::{
    CreateFileW, GetDriveTypeW, GetLogicalDrives, GetVolumeInformationW, OPEN_EXISTING,
};
use winapi::um::handleapi::CloseHandle;
use winapi::um::handleapi::INVALID_HANDLE_VALUE;
use winapi::um::ioapiset::DeviceIoControl;
use winapi::um::sysinfoapi::{GetSystemInfo, SYSTEM_INFO};
use winapi::um::winbase::DRIVE_FIXED;
use winapi::um::winioctl::{
     IOCTL_STORAGE_QUERY_PROPERTY,
};
use winapi::um::winnt::{BOOLEAN, FILE_SHARE_READ, FILE_SHARE_WRITE, HANDLE};

pub struct KeyHandler {
    pub unique_id: String,
    pub win_key: Vec<u16>,
}

impl KeyHandler {
    pub fn new(unique_id: String, win_key: Vec<u16>) -> KeyHandler {
        KeyHandler { unique_id, win_key }
    }
}

pub fn init_processors() -> (Vec<Processor>, String, String) {
    unsafe {
        let mut sys_info: SYSTEM_INFO = zeroed();
        GetSystemInfo(&mut sys_info);
        let (vendor_id, brand) = processor::get_vendor_id_and_brand(&sys_info);
        let frequencies = processor::get_frequencies(sys_info.dwNumberOfProcessors as usize);
        let mut ret = Vec::with_capacity(sys_info.dwNumberOfProcessors as usize + 1);
        for nb in 0..sys_info.dwNumberOfProcessors {
            ret.push(Processor::new_with_values(
                &format!("CPU {}", nb + 1),
                vendor_id.clone(),
                brand.clone(),
                frequencies[nb as usize],
            ));
        }
        (ret, vendor_id, brand)
    }
}


#[allow(non_snake_case)]
pub unsafe fn load_symbols() -> HashMap<String, u32> {
    use winapi::um::winreg::{RegQueryValueExA, HKEY_PERFORMANCE_DATA};

    let mut cbCounters = 0;
    let mut dwType = 0;
    let mut ret = HashMap::new();

    let _dwStatus = RegQueryValueExA(
        HKEY_PERFORMANCE_DATA,
        b"Counter 009\0".as_ptr() as *const _,
        ::std::ptr::null_mut(),
        &mut dwType as *mut i32 as *mut _,
        ::std::ptr::null_mut(),
        &mut cbCounters as *mut i32 as *mut _,
    );

    let mut lpmszCounters = Vec::with_capacity(cbCounters as usize);
    lpmszCounters.set_len(cbCounters as usize);
    let _dwStatus = RegQueryValueExA(
        HKEY_PERFORMANCE_DATA,
        b"Counter 009\0".as_ptr() as *const _,
        ::std::ptr::null_mut(),
        &mut dwType as *mut i32 as *mut _,
        lpmszCounters.as_mut_ptr(),
        &mut cbCounters as *mut i32 as *mut _,
    );
    for (pos, s) in lpmszCounters
        .split(|x| *x == 0)
        .filter(|x| !x.is_empty())
        .collect::<Vec<_>>()
        .chunks(2)
        .filter(|&x| x.len() == 2)
        .filter_map(
            |x| match (std::str::from_utf8(x[0]), String::from_utf8(x[1].to_vec())) {
                (Ok(n), Ok(s)) => {
                    if let Ok(n) = u32::from_str_radix(n, 10) {
                        Some((n, s))
                    } else {
                        None
                    }
                }
                _ => None,
            },
        )
    {
        ret.insert(s, pos as u32);
    }
    ret
}

#[allow(clippy::ptr_arg)]
pub fn get_translation(s: &String, map: &HashMap<String, u32>) -> Option<String> {
    use winapi::um::pdh::PdhLookupPerfNameByIndexW;

    if let Some(index) = map.get(s) {
        let mut size: usize = 0;
        unsafe {
            let _res = PdhLookupPerfNameByIndexW(
                ::std::ptr::null(),
                *index,
                ::std::ptr::null_mut(),
                &mut size as *mut usize as *mut _,
            );
            if size == 0 {
                return Some(String::new());
            } else {
                let mut v = Vec::with_capacity(size);
                v.set_len(size);
                let _res = PdhLookupPerfNameByIndexW(
                    ::std::ptr::null(),
                    *index,
                    v.as_mut_ptr() as *mut _,
                    &mut size as *mut usize as *mut _,
                );
                return Some(String::from_utf16(&v[..size - 1]).expect("invalid utf16"));
            }
        }
    }
    None
}

pub fn add_counter(
    s: String,
    query: &mut Query,
    keys: &mut Option<KeyHandler>,
    counter_name: String,
) {
    let mut full = s.encode_utf16().collect::<Vec<_>>();
    full.push(0);
    if query.add_counter(&counter_name, full.clone()) {
        *keys = Some(KeyHandler::new(counter_name, full));
    }
}
