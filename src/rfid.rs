use embassy_time::{Duration, Timer};
use esp_println::{print, println};
use mfrc522::Mfrc522;

pub const AUTH_SECTOR: u8 = 1; // Use any sector other than 0
pub const SECTOR_TRAILER: u8 = 3; //relative block within the sector (4th block within the sector 1)
pub const AUTH_DATA: [u8; 16] = [
    0x52, 0x75, 0x73, 0x74, 0x65, 0x64, // Key A: "Rusted"
    0xFF, 0x07, 0x80, 0x69, // Access bits and trailer byte
    0x46, 0x65, 0x72, 0x72, 0x69, 0x73, // Key B: "Ferris"
];
pub const DEFAULT_KEY: [u8; 6] = [0xFF; 6];

#[derive(Debug)]
pub enum RfidError {
    AuthFailed,
    WriteFailed,
    ReadFailed,
    CardSelectionFailed,
    HaltFailed,
    Unknown,
}

impl core::fmt::Display for RfidError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            RfidError::AuthFailed => write!(f, "Authentication failed"),
            RfidError::WriteFailed => write!(f, "Write operation failed"),
            RfidError::ReadFailed => write!(f, "Read operation failed"),
            RfidError::CardSelectionFailed => write!(f, "Card selection failed"),
            RfidError::HaltFailed => write!(f, "Halt failed"),
            RfidError::Unknown => write!(f, "Unknown error"),
        }
    }
}

pub struct Rfid<COMM: mfrc522::comm::Interface> {
    rc522: Mfrc522<COMM, mfrc522::Initialized>,
    uid: Option<mfrc522::Uid>,
}

impl<COMM: mfrc522::comm::Interface> Rfid<COMM> {
    pub fn new(rc522: Mfrc522<COMM, mfrc522::Initialized>) -> Self {
        Self { rc522, uid: None }
    }

    pub fn write_block(
        &mut self,
        sector: u8,
        rel_block: u8,
        data: [u8; 16],
        auth_key: &[u8; 6], //additional argument for the auth key
    ) -> Result<(), RfidError> {
        let uid = self.uid.as_ref().unwrap();
        let block_offset = sector * 4;
        let abs_block = block_offset + rel_block;

        self.rc522
            .mf_authenticate(uid, block_offset, auth_key)
            .map_err(|_| RfidError::AuthFailed)?;

        self.rc522
            .mf_write(abs_block, data)
            .map_err(|_| RfidError::WriteFailed)?;

        Ok(())
    }

    pub fn print_sector(&mut self, sector: u8, auth_key: &[u8; 6]) -> Result<(), RfidError> {
        let uid = self.uid.as_ref().unwrap();
        let block_offset = sector * 4;
        self.rc522
            .mf_authenticate(uid, block_offset, auth_key)
            .map_err(|_| RfidError::AuthFailed)?;

        for abs_block in block_offset..block_offset + 4 {
            let data = self
                .rc522
                .mf_read(abs_block)
                .map_err(|_| RfidError::ReadFailed)?;
            print_hex_bytes(&data);
        }

        Ok(())
    }

    pub fn get_trailer(&mut self, sector: u8, auth_key: &[u8; 6]) -> Result<[u8; 16], RfidError> {
        let uid = self.uid.as_ref().unwrap();
        let block_offset = sector * 4;
        self.rc522
            .mf_authenticate(uid, block_offset, auth_key)
            .map_err(|_| RfidError::AuthFailed)?;

        let sector_trailer = block_offset + SECTOR_TRAILER;
        let data = self
            .rc522
            .mf_read(sector_trailer)
            .map_err(|_| RfidError::ReadFailed)?;
        Ok(data)
    }

    pub async fn select_card(&mut self) -> bool {
        if let Ok(atqa) = self.rc522.reqa() {
            Timer::after(Duration::from_millis(50)).await;
            if let Ok(uid) = self.rc522.select(&atqa) {
                self.uid = Some(uid);
                return true;
            }
        }
        false
    }

    pub fn authenticate(&mut self) -> Result<(), RfidError> {
        let auth_key_a: &[u8; 6] = &AUTH_DATA[..6].try_into().unwrap(); // First 6 bytes of the block

        let trailer_data = self
            .get_trailer(AUTH_SECTOR, auth_key_a)
            .map_err(|_| RfidError::AuthFailed)?;

        let key_b = &trailer_data[10..];
        if key_b != &AUTH_DATA[10..] {
            return Err(RfidError::AuthFailed);
        }

        Ok(())
    }

    pub fn halt_state(&mut self) -> Result<(), RfidError> {
        self.rc522.hlta().map_err(|_| RfidError::HaltFailed)?;
        self.rc522
            .stop_crypto1()
            .map_err(|_| RfidError::HaltFailed)?;
        Ok(())
    }
}
pub fn print_hex_bytes(data: &[u8]) {
    for &b in data.iter() {
        print!("{:02x} ", b);
    }
    println!();
}
