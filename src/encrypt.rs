use log::{debug, info};

use openssl::rsa::{Padding, Rsa};

use std::fs;
use std::fs::File;
use std::io::{Read, Write};

use crate::Result;

// encoding blocks
const KEY_SIZE: usize = 1024;
const BLOCK_SIZE: usize = KEY_SIZE / 16;

pub fn get_file_as_byte_vec(filename: &String) -> Result<Vec<u8>> {
    let mut f = File::open(filename)?;
    let metadata = fs::metadata(filename)?;
    debug!("file size : {}", metadata.len());
    let mut buffer = vec![];
    f.read_to_end(&mut buffer)?;

    Ok(buffer)
}

/// encrypt file using a public key path
pub fn encrypt_file(filepath: &String, public_key_path: &String) -> Result<()> {
    info!("reading public key {}", public_key_path);
    let public_file_content = get_file_as_byte_vec(public_key_path)?;
    info!("public file content : {:?}", &public_file_content);

    let o = filepath.clone() + "x";
    encrypt_file_with_inmemory_key(filepath, &o, &public_file_content)?;
    Ok(())
}

pub fn check_public_key(public_key_content: &[u8]) -> Result<()> {
    let _rsa_key = Rsa::public_key_from_pem_pkcs1(public_key_content)?;
    info!("public key successfully read");
    Ok(())
}

/// encrypt file using an inmemory public key
pub fn encrypt_file_with_inmemory_key(
    filepath: &String,
    output_file: &String,
    public_key_content: &[u8],
) -> Result<()> {
    // read pem public file

    let rsa_key = Rsa::public_key_from_pem_pkcs1(public_key_content)?;

    debug!("{:?}", rsa_key);

    let filecontent = get_file_as_byte_vec(filepath)?;

    // to encrypt,
    let mut nbblock = filecontent.len() / BLOCK_SIZE;
    if nbblock * BLOCK_SIZE < filecontent.len() {
        nbblock += 1;
    }

    let mut outputfile = File::create(output_file)?;

    debug!("nb blocks : {}", nbblock);
    let binary_block_number = (nbblock as u32).to_le_bytes();
    outputfile.write_all(&binary_block_number)?;

    let mut _encrypt_size = 0;

    let mut left_to_encrypt = filecontent.len();
    for i in 0..nbblock {
        let bsize = std::cmp::min(left_to_encrypt, BLOCK_SIZE);
        let mut buffer = vec![0_u8; 2000];
        debug!("block {}, encoded block size : {} ", i, bsize);

        let slice: &[u8] = &filecontent[(BLOCK_SIZE * i)..(BLOCK_SIZE * i) + bsize];
        let crypted_buffer_size =
            rsa_key.public_encrypt(slice, &mut buffer, Padding::PKCS1_OAEP)?;
        _encrypt_size += crypted_buffer_size;
        left_to_encrypt -= bsize;

        debug!(
            "block {}, allocate {}, left = {}, buffer size = {}",
            i,
            crypted_buffer_size,
            left_to_encrypt,
            buffer.len()
        );
        let written_bytes = (crypted_buffer_size as u32).to_le_bytes();
        outputfile.write_all(&written_bytes)?;
        outputfile.write_all(&buffer[0..crypted_buffer_size])?;
    }
    outputfile.flush()?;
    Ok(())
}

/// decrypt file using a private key file
pub fn decrypt_file(
    filepath: String,
    private_key_path: String,
    passphrase: String,
    outputfilepath: String,
) -> Result<()> {
    let private_file_content = get_file_as_byte_vec(&private_key_path)?;

    let rsa_key =
        Rsa::private_key_from_pem_passphrase(&private_file_content, passphrase.as_bytes())?;

    debug!("{:?}", rsa_key);

    let mut result_file = fs::File::create(outputfilepath)?;

    let filecontent = get_file_as_byte_vec(&filepath)?;

    let mut cpt: usize = 0;
    let i_array: [u8; 4] = filecontent[0..4].try_into()?;
    let nbblocks = u32::from_le_bytes(i_array);
    cpt += 4;
    for i in 0..nbblocks {
        let i_array: [u8; 4] = filecontent[cpt..cpt + 4].try_into()?;
        cpt += 4;
        let sizeblock = u32::from_le_bytes(i_array);
        debug!("block {} size : {}", i, sizeblock);
        let mut buffer = vec![0_u8; 2000];

        let uncrypted_buffer_size = rsa_key.private_decrypt(
            &filecontent[cpt..cpt + (sizeblock as usize)],
            &mut buffer,
            Padding::PKCS1_OAEP,
        )?;

        debug!(" uncrypt buffer size : {}", uncrypted_buffer_size);

        result_file.write_all(&buffer[0..uncrypted_buffer_size])?;
        cpt += sizeblock as usize;
    }

    Ok(())
}
