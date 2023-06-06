/// These are possible interfaces that could be present on a device
#[derive(PartialEq)]
#[derive(Clone, Copy)]
pub enum Interface {
    /// Use the connected Software Defined Radio
    SDR,

    /// Use wireless chip
    WLAN,

    /// Use ethernet
    ETHER,

    System,
}