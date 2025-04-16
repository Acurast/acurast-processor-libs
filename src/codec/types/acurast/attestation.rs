use std::error::Error;
use std::io::Cursor;
use byteorder::{LittleEndian, ReadBytesExt};

use crate::cursor::{read_byte_array, read_list, CursorExt};

#[derive(uniffi::Record, Debug)]
pub struct Attestation {
    pub cert_ids: Vec<CertId>,
    pub content: AttestationContent,
    pub validity: AttestationValidity,
}

impl Attestation {
    pub fn read(
        mut cursor: &mut Cursor<Vec<u8>>
    ) -> Result<Self, Box<dyn Error>> {
        let cert_ids = read_list(&mut cursor, CertId::read)?;
        println!("Attestation:read: {:?}", cert_ids);
        let content = AttestationContent::read(&mut cursor)?;
        println!("Attestation:read content: {:?}", content);
        let validity = AttestationValidity::read(&mut cursor)?;

        Ok(Attestation {
            cert_ids,
            content,
            validity,
        })
    }
}

#[derive(uniffi::Record, Debug)]
pub struct AttestationValidity {
    pub not_before: u64,
    pub not_after: u64,
}

impl AttestationValidity {
    pub fn read(
        cursor: &mut Cursor<Vec<u8>>,
    ) -> Result<Self, Box<dyn Error>> {
        let not_before = cursor.read_u64::<LittleEndian>()?;
        let not_after = cursor.read_u64::<LittleEndian>()?;

        Ok(AttestationValidity {
            not_before,
            not_after,
        })
    }
}


#[derive(uniffi::Record, Debug)]
pub struct CertId {
    pub issuer_name: Vec<u8>,
    pub serial_number: Vec<u8>,
}

impl CertId {
    pub fn read(
        cursor: &mut Cursor<Vec<u8>>,
    ) -> Result<Self, Box<dyn Error>> {
        let issuer_name = read_byte_array(cursor)?;
        let serial_number = read_byte_array(cursor)?;

        Ok(CertId {
            issuer_name,
            serial_number,
        })
    }
}

#[derive(uniffi::Enum, Debug)]
pub enum AttestationContent {
    KeyDescription(KeyDescription),
    DeviceAttestation(DeviceAttestation),
}

impl AttestationContent {
    pub fn read(
        cursor: &mut Cursor<Vec<u8>>,
    ) -> Result<Self, Box<dyn Error>> {
        let id = cursor.read_u8()?;
        println!("AttestationContent:read: {:?}", id);
        match id {
            KeyDescription::ID => Ok(AttestationContent::KeyDescription(KeyDescription::read(cursor)?)),
            DeviceAttestation::ID => Ok(AttestationContent::DeviceAttestation(DeviceAttestation::read(cursor)?)),
            _ => Err(format!("Unknown AttestationContent ID: {}", id).into()),
        }
    }
}

#[derive(uniffi::Record, Debug)]
pub struct KeyDescription {
    pub attestation_security_level: u8,
    pub key_mint_security_level: u8,
}

impl KeyDescription {
    pub const ID: u8 = 0;

    pub fn read(
        cursor: &mut Cursor<Vec<u8>>,
    ) -> Result<Self, Box<dyn Error>> {
        let attestation_security_level = cursor.read_u8()?;
        let key_mint_security_level = cursor.read_u8()?;

        Ok(KeyDescription {
            attestation_security_level,
            key_mint_security_level,
        })
    }
}

#[derive(uniffi::Record, Debug)]
pub struct DeviceAttestation {
    pub key_usage_properties: KeyUsageProperties,
    pub device_os_information: DeviceOsInformation,
    pub nonce: Nonce,
}

impl DeviceAttestation {
    pub const ID: u8 = 1;

    pub fn read(cursor: &mut Cursor<Vec<u8>>) -> Result<Self, Box<dyn Error>> {
        let key_usage_properties = KeyUsageProperties::read(cursor)?;
        let device_os_information = DeviceOsInformation::read(cursor)?;
        let nonce = Nonce::read(cursor)?;

        Ok(DeviceAttestation {
            key_usage_properties,
            device_os_information,
            nonce,
        })
    }
}

#[derive(uniffi::Record, Debug)]
pub struct KeyUsageProperties {
    pub t4: Option<u64>,
    pub t1200: Option<u64>,
    pub t1201: Option<u64>,
    pub t1202: Option<u64>,
    pub t1203: Option<u64>,
    pub t1204: Option<Vec<u8>>,
    pub t5: Option<Vec<u8>>,
    pub t1206: Option<u64>,
    pub t1207: Option<u64>,
    pub t1209: Option<u64>,
    pub t1210: Option<u64>,
    pub t1211: Option<u64>,
}

impl KeyUsageProperties {
    pub fn read(cursor: &mut Cursor<Vec<u8>>) -> Result<Self, Box<dyn Error>> {
        let t4 = cursor.read_u64::<LittleEndian>().ok();
        let t1200 = cursor.read_u64::<LittleEndian>().ok();
        let t1201 = cursor.read_u64::<LittleEndian>().ok();
        let t1202 = cursor.read_u64::<LittleEndian>().ok();
        let t1203 = cursor.read_u64::<LittleEndian>().ok();
        let t1204 = cursor.read_optional(read_byte_array)?;
        let t5 = cursor.read_optional(read_byte_array)?;
        let t1206 = cursor.read_u64::<LittleEndian>().ok();
        let t1207 = cursor.read_u64::<LittleEndian>().ok();
        let t1209 = cursor.read_u64::<LittleEndian>().ok();
        let t1210 = cursor.read_u64::<LittleEndian>().ok();
        let t1211 = cursor.read_u64::<LittleEndian>().ok();

        Ok(KeyUsageProperties {
            t4,
            t1200,
            t1201,
            t1202,
            t1203,
            t1204,
            t5,
            t1206,
            t1207,
            t1209,
            t1210,
            t1211,
        })
    }
}

#[derive(uniffi::Record, Debug)]
pub struct DeviceOsInformation {
    pub t1400: Option<Vec<u8>>,
    pub t1104: Option<u64>,
    pub t1403: Option<Vec<u8>>,
    pub t1420: Option<Vec<u8>>,
    pub t1026: Option<Vec<u8>>,
    pub t1029: Option<Vec<u8>>,
}

impl DeviceOsInformation {
    pub fn read(cursor: &mut Cursor<Vec<u8>>) -> Result<Self, Box<dyn Error>> {
        let t1400 = cursor.read_optional(read_byte_array)?;
        let t1104 = cursor.read_u64::<LittleEndian>().ok();
        let t1403 = cursor.read_optional( read_byte_array)?;
        let t1420 = cursor.read_optional( read_byte_array)?;
        let t1026 = cursor.read_optional( read_byte_array)?;
        let t1029 = cursor.read_optional( read_byte_array)?;

        Ok(DeviceOsInformation {
            t1400,
            t1104,
            t1403,
            t1420,
            t1026,
            t1029,
        })
    }
}

#[derive(uniffi::Record, Debug)]
pub struct Nonce {
    pub nonce: Option<Vec<u8>>,
}

impl Nonce {
    pub fn read(cursor: &mut Cursor<Vec<u8>>) -> Result<Self, Box<dyn Error>> {
        let nonce = cursor.read_optional( read_byte_array)?;

        Ok(Nonce { nonce })
    }
}
