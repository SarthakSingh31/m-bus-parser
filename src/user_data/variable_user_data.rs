use super::data_information::FunctionField;
use super::data_information::{self, DataInformationField};
use super::value_information::{self, Unit, ValueInformation};
use super::DataRecords;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct DataRecord {
    pub function: FunctionField,
    pub storage_number: u64,
    pub unit: Unit,
    pub exponent: Exponent,
    pub quantity: Quantity,
    pub value: f64,
    pub size: usize,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Exponent {
    pub inner: Option<isize>,
}
impl From<isize> for Exponent {
    fn from(value: isize) -> Self {
        Exponent { inner: Some(value) }
    }
}

impl From<&ValueInformation> for Quantity {
    fn from(value_information: &ValueInformation) -> Quantity {
        match value_information {
            ValueInformation::Primary(x) => match x {
                0x00..=0x0F => Quantity::Energy,
                0x10..=0x17 => Quantity::Volume,
                0x18..=0x1F => Quantity::Mass,
                0x20..=0x27 => Quantity::Duration,
                0x28..=0x37 => Quantity::Power,
                0x38..=0x4F => Quantity::VolumeFlow,
                0x50..=0x57 => Quantity::MassFlow,
                0x58..=0x67 => Quantity::Temperature,
                0x68..=0x6B => Quantity::Pressure,
                0x6C..=0x6D => Quantity::TimePoint,
                0x74..=0x77 => Quantity::Duration,
                0x78 => Quantity::IdentificationNumber,
                _ => todo!("Implement the rest of the units: {:?}", x),
            },
            ValueInformation::PlainText(_) => Quantity::PlainText,
            ValueInformation::Extended(x) => match x {
                value_information::VIFExtension::DigitalInput => Quantity::BinaryDigitalInput,
                _ => todo!("Implement the rest of the units: {:?}", x),
            },
            ValueInformation::Any => todo!(),
            ValueInformation::ManufacturerSpecific => todo!(),
        }
    }
}

impl From<&ValueInformation> for Exponent {
    fn from(value_information: &ValueInformation) -> Exponent {
        match value_information {
            ValueInformation::Primary(x) => match x & 0x7F {
                0..=7 | 0x18..=0x1F | 0x28..=0x2F | 0x50..=0x57 => {
                    Exponent::from((x & 0b111) as isize - 3)
                }
                8..=15 | 0x30..=0x37 => Exponent::from((x & 0b111) as isize),
                0x10..=0x17 | 0x38..=0x3F | 0x6C..=0x6D => Exponent::from((x & 0b111) as isize - 6),
                0x20..=0x27 | 0x74..=0x77 => Exponent::from(1),
                0x40..=0x47 => Exponent::from((x & 0b111) as isize - 7),
                0x48..=0x4F => Exponent::from((x & 0b111) as isize - 9),
                0x58..=0x6B => Exponent::from((x & 0b11) as isize - 3),
                0x6E..=0x6F | 0x78 => Exponent { inner: None },
                _ => todo!("Implement the rest of the units: {:?}", x),
            },
            ValueInformation::PlainText(_) => Exponent { inner: None },
            ValueInformation::Extended(x) => match x {
                value_information::VIFExtension::DigitalInput => Exponent { inner: None },
                _ => todo!("Implement the rest of the units: {:?}", x),
            },
            ValueInformation::Any => todo!(),
            ValueInformation::ManufacturerSpecific => todo!(),
        }
    }
}

#[derive(Debug, Copy, PartialEq, Clone)]
pub enum Quantity {
    Volume,
    Energy,
    ManufacturerSpecific,
    ErrorFlags,
    TimePoint,
    VolumeFlow,
    MassFlow,
    Mass,
    Temperature,
    FlowTemperature,
    TemperatureDifference,
    BinaryDigitalInput,
    RelativeHumidity,
    AveragingDuration,
    ExternalTemperature,
    Duration,
    Power,
    Pressure,
    IdentificationNumber,
    PlainText,
}

#[derive(Debug, PartialEq)]
pub enum DataRecordError {
    DataInformationError(data_information::DataInformationError),
}

impl From<data_information::DataInformationError> for DataRecordError {
    fn from(error: data_information::DataInformationError) -> Self {
        DataRecordError::DataInformationError(error)
    }
}

impl From<value_information::ValueInformationError> for DataRecordError {
    fn from(_error: value_information::ValueInformationError) -> Self {
        DataRecordError::DataInformationError(data_information::DataInformationError::NoData)
    }
}

impl TryFrom<&[u8]> for DataRecord {
    type Error = DataRecordError;
    fn try_from(data: &[u8]) -> Result<DataRecord, DataRecordError> {
        let data_information = DataInformationField::try_from(data)?;
        let value_information = ValueInformation::try_from(&data[1..])?;
        let value_and_data_information_size =
            data_information.get_size() + value_information.get_size();
        match value_information {
            ValueInformation::PlainText(_) => {
                let plaintext_size = data[value_and_data_information_size] as usize;
                let total_size = value_and_data_information_size + plaintext_size + 1;
                Ok(DataRecord {
                    function: data_information.function_field,
                    storage_number: data_information.storage_number,
                    unit: Unit::try_from(&value_information)?,
                    exponent: Exponent::from(&value_information),
                    quantity: Quantity::from(&value_information),
                    value: 0.0,
                    size: total_size,
                })
            }
            _ => {
                let value = data_information
                    .data_field_coding
                    .extract_from_bytes(&data[value_and_data_information_size..]);
                let total_size = value_and_data_information_size + value.byte_size;
                Ok(DataRecord {
                    function: data_information.function_field,
                    storage_number: data_information.storage_number,
                    unit: Unit::try_from(&value_information)?,
                    exponent: Exponent::from(&value_information),
                    quantity: Quantity::from(&value_information),
                    value: value.data,
                    size: total_size,
                })
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum VariableUserDataError {
    DataInformationError(DataRecordError),
}

impl From<DataRecordError> for VariableUserDataError {
    fn from(error: DataRecordError) -> Self {
        VariableUserDataError::DataInformationError(error)
    }
}

impl TryFrom<&[u8]> for DataRecords {
    type Error = VariableUserDataError;
    fn try_from(data: &[u8]) -> Result<DataRecords, VariableUserDataError> {
        let mut records = DataRecords::new();
        let mut offset = 0;
        let mut _more_records_follow = false;

        while offset < data.len() {
            match data[offset] {
                0x0F => {
                    /* TODO: parse manufacturer specific */
                    offset = data.len();
                }
                0x1F => {
                    /* TODO: parse manufacturer specific */
                    _more_records_follow = true;
                    offset = data.len();
                }
                0x2F => {
                    offset += 1;
                }
                _ => {
                    let record = DataRecord::try_from(&data[offset..])?;
                    let _ = records.add_record(record);
                    offset += records.last().unwrap().size;
                }
            }
        }

        Ok(records)
    }
}

mod tests {

    #[test]
    fn test_parse_variable_data() {
        use crate::user_data::variable_user_data::Exponent;
        use crate::user_data::{
            data_information::FunctionField, value_information::Unit, variable_user_data::Quantity,
            DataRecord, DataRecords,
        };
        /* Data block 1: unit 0, storage No 0, no tariff, instantaneous volume, 12565 l (24 bit integer) */
        /* DIF = 0x03, VIF = 0x13, Value = 0x153100 */
        let data = &[0x03, 0x13, 0x15, 0x31, 0x00];

        let result = DataRecords::try_from(data.as_slice());
        assert_eq!(
            result.unwrap().get(0),
            Some(&DataRecord {
                function: FunctionField::InstantaneousValue,
                storage_number: 0,
                unit: Unit::CubicMeter,
                exponent: Exponent::from(-3),
                quantity: Quantity::Volume,
                value: 12565.0,
                size: 5,
            })
        );
    }

    #[test]
    fn test_parse_variable_data2() {
        use crate::user_data::variable_user_data::Exponent;
        use crate::user_data::{
            data_information::FunctionField, value_information::Unit, variable_user_data::Quantity,
            DataRecord, DataRecords,
        };
        /* Data block 2: unit 0, storage No 5, no tariff, maximum volume flow, 113 l/h (4 digit BCD) */
        let data = &[0x01, 0xFD, 0x1B, 0x00];

        let result = DataRecords::try_from(data.as_slice());
        assert_eq!(
            result.unwrap().get(0),
            Some(&DataRecord {
                function: FunctionField::InstantaneousValue,
                storage_number: 0,
                unit: Unit::WithoutUnits,
                exponent: Exponent { inner: None },
                quantity: Quantity::BinaryDigitalInput,
                value: 0.0,
                size: 4,
            })
        );
    }
    /*  Out: PlainText : Unit "%RH"  Value:   33.96
    In: 0x02, 0xFC, 0x03, 0x48, 0x52, 0x25, 0x74, 0x44, 0x0D*/
    #[test]
    fn test_parse_variable_data3() {
        use crate::user_data::variable_user_data::Exponent;
        use crate::user_data::{
            data_information::FunctionField, value_information::Unit, variable_user_data::Quantity,
            DataRecord, DataRecords,
        };
        /* Data block 3: unit 1, storage No 0, tariff 2, instantaneous energy, 218,37 kWh (6 digit BCD) */
        let data = &[0x02, 0xFC, 0x03, 0x48, 0x52, 0x25, 0x74, 0x44, 0x0D];

        let result = DataRecords::try_from(data.as_slice());
        assert_eq!(
            result.unwrap().get(0),
            Some(&DataRecord {
                function: FunctionField::InstantaneousValue,
                storage_number: 0,
                unit: Unit::PlainText,
                exponent: Exponent::from(-2),
                quantity: Quantity::PlainText,
                value: 33.96,
                size: 9,
            })
        );
    }

    fn _test_parse_variable_data2() {
        /* Data block 2: unit 0, storage No 5, no tariff, maximum volume flow, 113 l/h (4 digit BCD) */
        let _data = &[0xDA, 0x02, 0x3B, 0x13, 0x01];
    }

    fn _test_parse_variable_data3() {
        /* Data block 3: unit 1, storage No 0, tariff 2, instantaneous energy, 218,37 kWh (6 digit BCD) */
        let _data = &[0x8B, 0x60, 0x04, 0x37, 0x18, 0x02];
    }
}
