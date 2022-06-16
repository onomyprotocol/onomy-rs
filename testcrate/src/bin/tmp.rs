//! For temporary development testing

pub fn main() {
    /*
    use borsh::{BorshDeserialize, BorshSerialize};
    use equity_storage::EquityDatabase;
    use equity_types::Value;
    let database = EquityDatabase::in_memory();
    database.insert("testkey".as_bytes(), Value(1337)).unwrap();
    let res: Value = database.get("testkey".as_bytes()).unwrap().unwrap();
    let x: Vec<u8> = BorshSerialize::try_to_vec(&Value(1337)).unwrap();
    dbg!(&x);
    let res: Value = BorshDeserialize::try_from_slice(&x).unwrap();
    dbg!(res);
    */
}
