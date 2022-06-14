use crate::{
    data::input_file,
    error::{DatabaseSnafu, EnumValueSnafu, Error},
};
use rustbreak::{deser::Ron, FileDatabase};
use serde::{Deserialize, Serialize};
use snafu::ResultExt;
use std::{
    collections::{BTreeSet, HashMap},
    fmt::{Debug, Display},
    num::ParseIntError,
    ops::{Deref, DerefMut},
    path::PathBuf,
    str::FromStr,
};

#[derive(Debug)]
pub struct DriverDatabase {
    inner: FileDatabase<HardwareListing, Ron>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct HardwareListing {
    inner: HashMap<HardwareKind, DriverListing>,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash, clap::ArgEnum)]
pub enum HardwareKind {
    Graphics,
    Ethernet,
    Wireless,
    Sound,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DriverListing {
    inner: HashMap<BTreeSet<HardwareId>, Vec<DriverRecord>>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum HardwareId {
    Pci(PciId),
    Usb(UsbId),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DriverRecord {
    pub name: String,
    pub description: String,
    pub tags: Vec<String>,
    pub packages: Vec<String>,
    pub configurations: Vec<ConfigurationRecord>,
    pub pre_install_script: Option<Script>,
    pub post_install_script: Option<Script>,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PciId {
    pub vendor_id: u16,
    pub device_id: u16,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UsbId {
    pub vendor_id: u16,
    pub device_id: u16,
}

#[derive(
    Default,
    Debug,
    PartialEq, // Required to implement Eq
    Eq,        // Required by RangeInclusiveMap to implement Serialize and Deserialize
    Clone,
    Serialize,
    Deserialize,
)]
pub struct ConfigurationRecord {
    pub format: ConfigurationFormat,
    pub path: PathBuf,
    pub entry_map: HashMap<String, String>,
}

#[derive(
    Debug,
    PartialEq, // Required to implement Eq
    Eq,        // Required by RangeInclusiveMap to implement Serialize and Deserialize
    Clone,     // Required by RangeInclusiveMap to implement Serialize and Deserialize
    Serialize,
    Deserialize,
)]
pub enum ConfigurationFormat {
    Ini,
    Json,
    Yaml,
    Toml,
    Xml,
}

#[derive(
    Default,
    Debug,
    PartialEq, // Required to implement Eq
    Eq,        // Required by RangeInclusiveMap to implement Serialize and Deserialize
    Clone,     // Required by RangeInclusiveMap to implement Serialize and Deserialize
    Serialize,
    Deserialize,
)]
pub struct Script {
    pub path: PathBuf,
    pub script_kind: ScriptKind,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum ScriptKind {
    Python,
    JavaScript,
    Shell,
}

#[derive(Clone, Debug)]
pub enum ParsePciIdError {
    InvalidVendorId(ParseIntError),
    InvalidDeviceId(ParseIntError),
}

#[derive(Clone, Debug)]
pub enum ParseUsbIdError {
    InvalidVendorId(ParseIntError),
    InvalidDeviceId(ParseIntError),
}

impl DriverDatabase {
    pub fn load_with_database_path(filepath: PathBuf) -> Result<Self, Error> {
        Ok(DriverDatabase {
            inner: FileDatabase::<HardwareListing, Ron>::load_from_path_or_default(filepath)
                .context(DatabaseSnafu {})?,
        })
    }

    pub fn create_with_database_path(filepath: PathBuf) -> Result<Self, Error> {
        Ok(DriverDatabase {
            inner: FileDatabase::<HardwareListing, Ron>::create_at_path(
                filepath,
                HardwareListing::default(),
            )
            .context(DatabaseSnafu {})?,
        })
    }
}

impl Deref for DriverDatabase {
    type Target = FileDatabase<HardwareListing, Ron>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for DriverDatabase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl HardwareListing {
    pub fn new() -> Self {
        Self {
            inner: HashMap::<HardwareKind, DriverListing>::new(),
        }
    }

    pub fn all_packages(&self) -> HashMap<HardwareKind, Vec<String>> {
        self.iter()
            .map(|hardware_entry| (hardware_entry.0.to_owned(), hardware_entry.1.all_packages()))
            .collect()
    }
}

impl Deref for HardwareListing {
    type Target = HashMap<HardwareKind, DriverListing>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for HardwareListing {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl HardwareKind {
    pub fn all_to_strings() -> Vec<String> {
        vec![
            String::from("graphics"),
            String::from("ethernet"),
            String::from("wireless"),
            String::from("sound"),
        ]
    }
}

impl Default for HardwareListing {
    fn default() -> Self {
        Self {
            inner: Default::default(),
        }
    }
}

impl Default for HardwareKind {
    fn default() -> Self {
        HardwareKind::Graphics
    }
}

impl Display for HardwareKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            &HardwareKind::Graphics => write!(f, "Graphics"),
            &HardwareKind::Ethernet => write!(f, "Ethernet"),
            &HardwareKind::Wireless => write!(f, "Wireless"),
            &HardwareKind::Sound => write!(f, "Sound"),
        }
    }
}

impl TryFrom<&str> for HardwareKind {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let hardware_kind = match value.to_lowercase().as_ref() {
            "graphics" => HardwareKind::Graphics,
            "ethernet" => HardwareKind::Ethernet,
            "wireless" => HardwareKind::Wireless,
            "sound" => HardwareKind::Sound,
            _ => EnumValueSnafu {
                value: value.to_string(),
                enum_name: "HardwareKind".to_string(),
                allowed_values: Self::all_to_strings(),
            }
            .fail()?,
        };
        Ok(hardware_kind)
    }
}

impl TryFrom<String> for HardwareKind {
    type Error = Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_ref())
    }
}

impl FromStr for HardwareKind {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(s)
    }
}

impl From<input_file::HardwareKind> for HardwareKind {
    fn from(hardware_kind: input_file::HardwareKind) -> Self {
        match hardware_kind {
            input_file::HardwareKind::Graphics => HardwareKind::Graphics,
            input_file::HardwareKind::Ethernet => HardwareKind::Ethernet,
            input_file::HardwareKind::Wireless => HardwareKind::Wireless,
            input_file::HardwareKind::Sound => HardwareKind::Sound,
        }
    }
}

impl DriverListing {
    pub fn new() -> Self {
        Self {
            inner: HashMap::<BTreeSet<HardwareId>, Vec<DriverRecord>>::new(),
        }
    }

    pub fn all_packages(&self) -> Vec<String> {
        let mut packages = Vec::<String>::new();
        for hardware_id_entry in self.iter() {
            for driver_record in hardware_id_entry.1 {
                packages.extend(driver_record.packages.to_owned().into_iter());
            }
        }
        packages
    }
}

impl Deref for DriverListing {
    type Target = HashMap<BTreeSet<HardwareId>, Vec<DriverRecord>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for DriverListing {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl Default for DriverListing {
    fn default() -> Self {
        Self {
            inner: Default::default(),
        }
    }
}

impl PciId {
    pub fn new(vendor_id: u16, device_id: u16) -> Self {
        Self {
            vendor_id: vendor_id,
            device_id: device_id,
        }
    }

    pub fn vendor_id(&self) -> u16 {
        self.vendor_id
    }

    pub fn device_id(&self) -> u16 {
        self.device_id
    }
}

impl Display for PciId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:04x}:{:04x}", self.vendor_id, self.device_id)
    }
}

impl Debug for PciId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PciId")
            .field("vendor_id", &format!("{:04x}", &self.vendor_id))
            .field("device_id", &format!("{:04x}", &self.device_id))
            .finish()
    }
}

impl TryFrom<(&str, &str)> for PciId {
    type Error = ParsePciIdError;

    fn try_from(value: (&str, &str)) -> Result<Self, Self::Error> {
        let (vendor_id, device_id) = value;
        let vendor_id = u16::from_str_radix(vendor_id, 16)
            .map_err(|parse_int_error| ParsePciIdError::InvalidVendorId(parse_int_error))?;
        let device_id = u16::from_str_radix(device_id, 16)
            .map_err(|parse_int_error| ParsePciIdError::InvalidDeviceId(parse_int_error))?;
        Ok(Self::new(vendor_id, device_id))
    }
}

impl TryFrom<(String, String)> for PciId {
    type Error = ParsePciIdError;

    fn try_from(value: (String, String)) -> Result<Self, Self::Error> {
        let (vendor_id, device_id) = value;
        PciId::try_from((vendor_id.as_ref(), device_id.as_ref()))
    }
}

impl Display for ParsePciIdError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParsePciIdError::InvalidVendorId(parse_int_error) => {
                write!(f, "Invalid Vendor ID. Please refer to {}", parse_int_error)
            }
            ParsePciIdError::InvalidDeviceId(parse_int_error) => {
                write!(f, "Invalid Device ID. Please refer to {}", parse_int_error)
            }
        }
    }
}

impl UsbId {
    pub fn new(vendor_id: u16, device_id: u16) -> Self {
        Self {
            vendor_id: vendor_id,
            device_id: device_id,
        }
    }

    pub fn vendor_id(&self) -> u16 {
        self.vendor_id
    }

    pub fn device_id(&self) -> u16 {
        self.device_id
    }
}

impl Display for UsbId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:04x}:{:04x}", self.vendor_id, self.device_id)
    }
}

impl Debug for UsbId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UsbId")
            .field("vendor_id", &format!("{:04x}", &self.vendor_id))
            .field("device_id", &format!("{:04x}", &self.device_id))
            .finish()
    }
}

impl TryFrom<(&str, &str)> for UsbId {
    type Error = ParseUsbIdError;

    fn try_from(value: (&str, &str)) -> Result<Self, Self::Error> {
        let (vendor_id, device_id) = value;
        let vendor_id = u16::from_str_radix(vendor_id, 16)
            .map_err(|parse_int_error| ParseUsbIdError::InvalidVendorId(parse_int_error))?;
        let device_id = u16::from_str_radix(device_id, 16)
            .map_err(|parse_int_error| ParseUsbIdError::InvalidDeviceId(parse_int_error))?;
        Ok(Self::new(vendor_id, device_id))
    }
}

impl TryFrom<(String, String)> for UsbId {
    type Error = ParseUsbIdError;

    fn try_from(value: (String, String)) -> Result<Self, Self::Error> {
        let (vendor_id, device_id) = value;
        UsbId::try_from((vendor_id.as_ref(), device_id.as_ref()))
    }
}

impl Display for ParseUsbIdError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseUsbIdError::InvalidVendorId(parse_int_error) => {
                write!(f, "Invalid Vendor ID. Please refer to {}", parse_int_error)
            }
            ParseUsbIdError::InvalidDeviceId(parse_int_error) => {
                write!(f, "Invalid Device ID. Please refer to {}", parse_int_error)
            }
        }
    }
}

impl From<input_file::Configuration> for ConfigurationRecord {
    fn from(input_configuration: input_file::Configuration) -> Self {
        ConfigurationRecord {
            format: input_configuration.format.into(),
            path: input_configuration.path,
            entry_map: input_configuration.entry_map,
        }
    }
}

impl Default for ConfigurationFormat {
    fn default() -> Self {
        return ConfigurationFormat::Ini;
    }
}

impl From<input_file::ConfigurationFormat> for ConfigurationFormat {
    fn from(input_configuration_format: input_file::ConfigurationFormat) -> Self {
        match input_configuration_format {
            input_file::ConfigurationFormat::Ini => ConfigurationFormat::Ini,
            input_file::ConfigurationFormat::Json => ConfigurationFormat::Json,
            input_file::ConfigurationFormat::Yaml => ConfigurationFormat::Yaml,
            input_file::ConfigurationFormat::Toml => ConfigurationFormat::Toml,
            input_file::ConfigurationFormat::Xml => ConfigurationFormat::Xml,
        }
    }
}

impl From<input_file::Script> for Script {
    fn from(input_script: input_file::Script) -> Self {
        Script {
            path: input_script.path,
            script_kind: input_script.language.into(),
        }
    }
}

impl Default for ScriptKind {
    fn default() -> Self {
        return Self::Shell;
    }
}

impl From<input_file::ScriptKind> for ScriptKind {
    fn from(input_script_kind: input_file::ScriptKind) -> Self {
        match input_script_kind {
            input_file::ScriptKind::Python => ScriptKind::Python,
            input_file::ScriptKind::JavaScript => ScriptKind::JavaScript,
            input_file::ScriptKind::Shell => ScriptKind::Shell,
        }
    }
}
