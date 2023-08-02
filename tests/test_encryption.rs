#[cfg(test)]

mod test_encryption {
    // Note this useful idiom: importing names from outer (for mod tests) scope.

    use encrypter::encrypt::*;

    fn encrypt_decrypt(midi_file: String) {
        println!("encrypt midi file {}", &midi_file);
        encrypt_file(&midi_file.clone(), &"test_public.key.pem".into()).expect("fail to encrypt");

        decrypt_file(
            midi_file.clone() + "x".into(),
            "test_private.key.pem".into(),
            "30d9690cc085429a1d0a3ae787932bf1518a1798".into(),
            "result".into(),
        )
        .expect("fail to decrypt");

        // check result
        let src = get_file_as_byte_vec(&midi_file).expect("cannot read the source file");
        let dest =
            get_file_as_byte_vec(&"result".into()).expect("cannot read the destination file");
        assert!(src.len() == dest.len());
        for (index, i) in src.iter().enumerate() {
            assert!(*i == dest[index]);
        }
    }
    #[test]
    fn test_encrypt() {
        env_logger::init();

        let midi_file: String = "lalala1.mid".into();
        encrypt_decrypt(midi_file);
        encrypt_decrypt("113-BennyHill.mid".into());
    }
}
