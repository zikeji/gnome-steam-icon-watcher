/*
 * This file is a Copilot assisted Rust conversion and adaptation of logic from:
 *   - SteamDatabase/SteamAppInfo (https://github.com/SteamDatabase/SteamAppInfo), MIT License
 *   - ValveResourceFormat/ValveKeyValue (https://github.com/ValveResourceFormat/ValveKeyValue), MIT License
 *
 * Copyright (c) 2025 Zikeji
 *
 * This file and the project are licensed under the MIT License.
 * See the LICENSE file in the project root for license details.
 */

use byteorder::{LittleEndian, ReadBytesExt};
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::PathBuf;

const BIN_NONE: u8 = 0x00;
const BIN_STRING: u8 = 0x01;
const BIN_INT32: u8 = 0x02;
const BIN_END: u8 = 0x08;

#[derive(Debug, Clone)]
enum VdfValue {
    String(String),
    Int(i32),
    Object(HashMap<String, VdfValue>),
}

fn read_string<R: Read>(reader: &mut R) -> io::Result<String> {
    let mut buf = Vec::with_capacity(32);
    let mut byte = [0u8; 1];
    loop {
        reader.read_exact(&mut byte)?;
        if byte[0] == 0 {
            break;
        }
        buf.push(byte[0]);
    }
    String::from_utf8(buf).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

fn parse_binary_vdf<R: Read + Seek>(
    reader: &mut R,
    key_table: &Option<&[String]>,
) -> io::Result<HashMap<String, VdfValue>> {
    fn parse_object<R: Read + Seek>(
        reader: &mut R,
        key_table: &Option<&[String]>,
    ) -> io::Result<(HashMap<String, VdfValue>, bool)> {
        let mut result = HashMap::new();
        loop {
            let mut type_byte = [0u8; 1];
            if reader.read_exact(&mut type_byte).is_err() {
                return Ok((result, true));
            }
            if type_byte[0] == BIN_END {
                return Ok((result, false));
            }
            let key = if let Some(table) = key_table {
                let idx = reader.read_i32::<LittleEndian>()?;
                if idx < 0 || idx as usize >= table.len() {
                    format!("unknown_{}", idx)
                } else {
                    table[idx as usize].clone()
                }
            } else {
                read_string(reader)?
            };
            match type_byte[0] {
                BIN_NONE => {
                    let (obj, eof) = parse_object(reader, key_table)?;
                    result.insert(key, VdfValue::Object(obj));
                    if eof {
                        return Ok((result, true));
                    }
                }
                BIN_STRING => {
                    let value = read_string(reader)?;
                    result.insert(key, VdfValue::String(value));
                }
                BIN_INT32 => {
                    let value = reader.read_i32::<LittleEndian>()?;
                    result.insert(key, VdfValue::Int(value));
                }
                _ => {
                    if let Err(_) = read_string(reader) {
                        let _ = reader.read_i32::<LittleEndian>();
                    }
                }
            }
        }
    }
    let (result, _) = parse_object(reader, key_table)?;
    Ok(result)
}

fn parse_appinfo_file(
    path: &PathBuf,
    stop_at_app_id: Option<i32>,
) -> io::Result<Option<HashMap<String, VdfValue>>> {
    let mut file = File::open(path)?;
    let mut magic = [0u8; 4];
    file.read_exact(&mut magic)?;
    let _universe = file.read_u32::<LittleEndian>()?;
    let key_table_offset = file.read_i64::<LittleEndian>()?;
    let current_pos = file.seek(SeekFrom::Current(0))?;
    file.seek(SeekFrom::Start(key_table_offset as u64))?;
    let key_count = file.read_i32::<LittleEndian>()?;
    let mut key_table = Vec::with_capacity(key_count as usize);
    for _ in 0..key_count {
        let key = read_string(&mut file)?;
        key_table.push(key);
    }
    file.seek(SeekFrom::Start(current_pos))?;
    while file.seek(SeekFrom::Current(0))? < key_table_offset as u64 - 4 {
        let app_id = file.read_u32::<LittleEndian>()?;
        let entry_size = file.read_u32::<LittleEndian>()?;
        let _infostate = file.read_u32::<LittleEndian>()?;
        let _last_updated = file.read_u32::<LittleEndian>()?;
        let _access_token = file.read_u64::<LittleEndian>()?;
        let mut sha_hash = [0u8; 20];
        file.read_exact(&mut sha_hash)?;
        let _change_number = file.read_u32::<LittleEndian>()?;
        let mut vdf_sha_hash = [0u8; 20];
        file.read_exact(&mut vdf_sha_hash)?;
        let header_size = 4 + 4 + 4 + 4 + 8 + 20 + 4 + 20;
        let vdf_section_size = entry_size as usize - (header_size - 8);
        let section_start = file.seek(SeekFrom::Current(0))?;
        let mut raw_data = vec![0u8; vdf_section_size];
        file.read_exact(&mut raw_data)?;
        let mut cursor = std::io::Cursor::new(&raw_data);
        match parse_binary_vdf(&mut cursor, &Some(&key_table)) {
            Ok(vdf_data) => {
                if let Some(target_app_id) = stop_at_app_id {
                    if app_id == target_app_id as u32 {
                        return Ok(Some(vdf_data));
                    }
                }
            }
            Err(_) => {}
        }
        file.seek(SeekFrom::Start(section_start + vdf_section_size as u64))?;
    }
    Ok(None)
}

pub fn get_clienticon_from_appinfo(app_id: &str, path: &PathBuf) -> io::Result<Option<String>> {
    if !path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("appinfo.vdf not found at {:?}", path),
        ));
    }
    if let Ok(app_id) = app_id.parse::<i32>() {
        if let Some(section) = parse_appinfo_file(path, Some(app_id))? {
            if let Some(VdfValue::Object(appinfo)) = section.get("appinfo") {
                if let Some(VdfValue::Int(id)) = appinfo.get("appid") {
                    if *id == app_id {
                        if let Some(VdfValue::Object(common)) = appinfo.get("common") {
                            if let Some(VdfValue::String(clienticon)) = common.get("clienticon") {
                                return Ok(Some(clienticon.clone()));
                            }
                        }
                        return Ok(None);
                    }
                }
            }
        }
    }
    Ok(None)
}
