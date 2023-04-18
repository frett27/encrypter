#[cfg(test)]

mod test_db {
    // Note this useful idiom: importing names from outer (for mod tests) scope.

    use encrypter::keys_management::*;

    #[test]
    fn test_db_keys() {
        let d = Database::open_database().expect("fail to open database");

        let k = Key {
            rowid: 0,
            name: "hello".into(),
            sha1: "kk".into(),
            public_key: Some("hello".as_bytes().to_vec()),
        };

        d.insert(&k).expect("fail to insert key");

        for k in d.get_all() {
            println!("key : {:?}", k);
        }
    }
}
