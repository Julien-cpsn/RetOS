pub const NICS: [NIC;2] = [
    NIC { model: "e1000", mac: "52:54:00:12:34:56", tap: Some("tap0") },
    NIC { model: "rtl8139", mac: "52:54:00:12:34:57", tap: Some("tap1") },
];
pub const TELNET: &str = "127.0.0.1:5000";

pub struct NIC {
    pub model: &'static str,
    pub mac: &'static str,
    pub tap: Option<&'static str>,
}