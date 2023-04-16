use openssl::rsa::{Padding, Rsa};

use std::fs;
use std::fs::File;
use std::io::{Read, Result, Write};

const BLOCK_SIZE: usize = 4096 / 16;

fn get_file_as_byte_vec(filename: &String) -> Vec<u8> {
    let mut f = File::open(&filename).expect("no file found");
    let metadata = fs::metadata(&filename).expect("unable to read metadata");
    let mut buffer = vec![0; metadata.len() as usize];
    f.read(&mut buffer).expect("buffer overflow");

    buffer
}

pub fn encrypt_file(filepath: String, public_key_path: String) {
    // read pem public file

    let public_file_content = get_file_as_byte_vec(&public_key_path);

    let rsa_key =
        Rsa::public_key_from_pem_pkcs1(&public_file_content).expect("fail to read pem file");

    println!("{:?}", rsa_key);

    let filecontent = get_file_as_byte_vec(&filepath);

    // to encrypt,
    let mut nbblock = filecontent.len() / BLOCK_SIZE;
    if nbblock * BLOCK_SIZE < filecontent.len() {
        nbblock += 1;
    }

    let mut outputfile = File::create(filepath + "x".into()).expect("cannot write file");

    println!("nb blocks : {}", nbblock);
    let binary_block_number = (nbblock as u32).to_le_bytes().clone();
    outputfile
        .write(&binary_block_number)
        .expect("fail to write block count");

    let mut encrypt_size = 0;

    let mut left_to_encrypt = filecontent.len();
    for i in 0..nbblock {
        
        let bsize = std::cmp::min(left_to_encrypt, BLOCK_SIZE);
        let mut buffer = vec![0_u8; 2000];
        println!("block {}, encoded block size : {} ", i, bsize);

        let slice: &[u8] = &filecontent[(BLOCK_SIZE * i)..(BLOCK_SIZE * i) + bsize];
        let crypted_buffer_size = rsa_key
            .public_encrypt(&slice, &mut buffer, Padding::PKCS1_OAEP)
            .expect("fail to crypt");
        encrypt_size += crypted_buffer_size;
        left_to_encrypt -= bsize;

        println!(
            "block {}, allocate {}, left = {}, buffer size = {}",
            i,
            crypted_buffer_size,
            left_to_encrypt,
            buffer.len()
        );
        let written_bytes = (crypted_buffer_size as u32).to_le_bytes().clone();
        outputfile
            .write(&written_bytes)
            .expect("fail to write elements");
        outputfile
            .write(&buffer[0..crypted_buffer_size])
            .expect("fail to write buffer");
    }
    outputfile.flush();
}

pub fn decrypt_file(filepath: String, private_key_path: String, passphrase: String) {
    let private_file_content = get_file_as_byte_vec(&private_key_path);

    let rsa_key =
        Rsa::private_key_from_pem_passphrase(&private_file_content, &passphrase.as_bytes())
        .expect("fail to read pem file");

    println!("{:?}", rsa_key);

    let mut result_file = fs::File::create("result").expect("fail to create file");

    let filecontent = get_file_as_byte_vec(&filepath);

    let mut cpt : usize = 0;
    let i_array : [u8;4] = filecontent[0..4].try_into().expect("error reading");
    let nbblocks = u32::from_le_bytes(i_array);
    cpt +=4;
    for i in 0..nbblocks {

        let i_array : [u8;4] = filecontent[cpt..cpt+4].try_into().expect("error reading");
        cpt += 4;
        let sizeblock = u32::from_le_bytes(i_array);
        println!("block {} size : {}", i, sizeblock);
        let mut buffer = vec![0_u8; 2000];

        let uncrypted_buffer_size = rsa_key
        .private_decrypt(&filecontent[cpt..cpt+(sizeblock as usize)], &mut buffer, Padding::PKCS1_OAEP)
        .expect("fail to crypt");

        println!(" uncrypt buffer size : {}", uncrypted_buffer_size);


        result_file.write(&buffer[0..uncrypted_buffer_size]).expect("error inwriting to output file");
        cpt += sizeblock as usize;
    }


}



