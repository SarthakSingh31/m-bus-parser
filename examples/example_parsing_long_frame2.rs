use m_bus_parser::frames::{Address, Frame, Function};
fn main() {
    let example = vec![
        0x68, 0x3C, 0x3C, 0x68, 0x08, 0x08, 0x72, 0x78, 0x03, 0x49, 0x11, 0x77, 0x04, 0x0E, 0x16,
        0x0A, 0x00, 0x00, 0x00, 0x0C, 0x78, 0x78, 0x03, 0x49, 0x11, 0x04, 0x13, 0x31, 0xD4, 0x00,
        0x00, 0x42, 0x6C, 0x00, 0x00, 0x44, 0x13, 0x00, 0x00, 0x00, 0x00, 0x04, 0x6D, 0x0B, 0x0B,
        0xCD, 0x13, 0x02, 0x27, 0x00, 0x00, 0x09, 0xFD, 0x0E, 0x02, 0x09, 0xFD, 0x0F, 0x06, 0x0F,
        0x00, 0x01, 0x75, 0x13, 0xD3, 0x16,
    ];
    let frame = Frame::try_from(example.as_slice()).unwrap();

    if let Frame::LongFrame {
        function,
        address,
        data,
    } = frame
    {
        assert_eq!(
            function,
            Function::RspUd {
                acd: false,
                dfc: false
            }
        );
        assert_eq!(address, Address::Primary(8));

        if let Ok(m_bus_parser::user_data::UserDataBlock::VariableDataStructure {
            fixed_data_header,
            variable_data_block,
        }) = m_bus_parser::user_data::UserDataBlock::try_from(data)
        {
            println!("fixed_data_header: {:?}", fixed_data_header);
            println!("variable_data_block: {:?}", variable_data_block);
            let data_records = m_bus_parser::user_data::DataRecords::try_from(variable_data_block);
            assert!(data_records.is_ok());
            println!("data_records: {:#?}", data_records.unwrap());
        }
    }
}
