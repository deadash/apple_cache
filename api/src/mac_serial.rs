

#[derive(Default)]
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
    pub gq_serial: String,
    pub fy_serial: String,
    pub kb_serial: String,
    pub oy_serial: String,
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

    pub fn init(&mut self)
    {
        // sysctl -n kern.osversion
        self.osversion = "19F96\0".to_owned();
        // sysctl -n kern.osrevision
        self.osrevision = 199506;
        self.board_id = "3434304258204465736B746F70205265666572656E636520506C6174666F726D00\0".to_owned();
        self.product_name = "564d77617265372e3100\0".to_owned();
        // ioreg -l -p IODeviceTree | grep [system or boot]-uuid
        self.boot_uuid = "45453146324344302D364335422D343844332D393741392D39393634413832413430394500\0".to_owned();
        self.serial_number = "VMTesmDVV4yX\0".to_owned();
        self.uuid = "564D4DEB-260D-5578-C978-B87AA54123BA\0".to_owned();
        self.gq_serial = "8129849DE39952B4D842E30D9DA8C7E65D\0".to_owned();
        self.fy_serial = "BB85DA6FFAFC55FB18CBCF44A8FFE124F7\0".to_owned();
        self.kb_serial = "365F8ECD4B60A2C2AAC8D7F68407F832C4\0".to_owned();
        // ifconfig en0 | awk '/ether/{ gsub(":",""); print $2 }'
        self.mac_address = "000c294123ba\0".to_owned();
        self.oy_serial = "C691B3B001E13719C628572CB5C4DFFAFB\0".to_owned();
        self.ab_serial = "5B502BFA6BA9483189A297C29445030FCC\0".to_owned();
        self.rom = "564D4DEB260D\0".to_owned();
        self.mlb = "56586A4A654C68367055456A75672E2E2E\0".to_owned();
    }
}