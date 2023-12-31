use std::fs;
use serde::Deserialize;

use anyhow::Result;

#[derive(Default, Deserialize, Debug)]
pub(crate)
struct MacSerial
{
    pub osversion: String,
    pub osrevision: u32,
    pub board_id: String,
    pub product_name: String,
    pub boot_uuid: String,
    pub serial_number: String,
    pub uuid: String,
    pub mac_address: String,
    pub rom: String,
    pub mlb: String,
    #[serde(rename = "Gq3489ugfi")]
    pub gq_serial: String,
    #[serde(rename = "Fyp98tpgj")]
    pub fy_serial: String,
    #[serde(rename = "kbjfrfpoJU")]
    pub kb_serial: String,
    #[serde(rename = "oycqAZloTNDm")]
    pub oy_serial: String,
    #[serde(rename = "abKPld1EcMni")]
    pub ab_serial: String,
}

impl MacSerial
{
    fn new() -> Self
    {
        MacSerial { .. Default::default()  }
    }

    pub fn instance() -> &'static mut MacSerial {
        static mut MACSERIAL: Option<MacSerial> = None;
        unsafe {
           match MACSERIAL {
               Some(ref mut s) => s,
               None => {
                    let s = MacSerial::new();
                    MACSERIAL = Some(s);
                    MACSERIAL.as_mut().unwrap()
               }
           }
        }
    }

    fn zero(s: String) -> String
    {
        format!("{}\x00", s)
    }

    pub fn init(&mut self) -> Result<()>
    {
        let s = fs::read("mac.toml")?;
        let conf: MacSerial = toml::from_slice(&s)?;

        self.copy_from_str(conf)
    }

    fn copy_from_str(&mut self, conf: MacSerial) -> Result<()> {
        self.osversion = Self::zero(conf.osversion);
        self.osrevision = conf.osrevision;

        self.board_id = Self::zero(hex::encode(Self::zero(conf.board_id)));
        self.product_name = Self::zero(hex::encode(Self::zero(conf.product_name)));
        self.boot_uuid = Self::zero(hex::encode(Self::zero(conf.boot_uuid)));
        self.serial_number = Self::zero(conf.serial_number);
        self.uuid = Self::zero(conf.uuid);
        self.mac_address = Self::zero(conf.mac_address);
        self.rom = Self::zero(conf.rom);
        self.mlb = Self::zero(hex::encode(conf.mlb));
        self.gq_serial = Self::zero(conf.gq_serial.to_lowercase());
        self.fy_serial = Self::zero(conf.fy_serial.to_lowercase());
        self.kb_serial = Self::zero(conf.kb_serial.to_lowercase());
        self.oy_serial = Self::zero(conf.oy_serial.to_lowercase());
        self.ab_serial = Self::zero(conf.ab_serial.to_lowercase());

        Ok(())
    }

    pub fn init_from_json(&mut self, json: &str) -> Result<()> {
        let conf: MacSerial = serde_json::from_str(json)?;
        self.copy_from_str(conf)
    }
}