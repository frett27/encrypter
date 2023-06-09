#[cfg(test)]

mod test_encryption {
    // Note this useful idiom: importing names from outer (for mod tests) scope.

    use encrypter::encrypt::*;

    #[test]
    fn test_encrypt() {
        let midi_file: String = "lalala1.mid".into();
        println!("encrypt midi file {}", &midi_file);
        encrypt_file(&midi_file.clone(), &"test_public.key.pem".into()).expect("fail to encrypt");

        decrypt_file(
            midi_file + "x".into(),
            "test_private.key.pem".into(),
            "30d9690cc085429a1d0a3ae787932bf1518a1798".into(),
            "result".into(),
        )
        .expect("fail to decrypt");
    }

    #[test]
    fn test_encrypt2() {
        let midi_file: String = "t".into();
        println!("encrypt midi file {}", &midi_file);
        encrypt_file(&midi_file.clone(), &"test_public.key.pem".into()).expect("fail to encrypt");

        decrypt_file(
            midi_file + "x".into(),
            "test_private.key.pem".into(),
            "30d9690cc085429a1d0a3ae787932bf1518a1798".into(),
            "result".into(),
        )
        .expect("fail to encrypt");
    }
}
