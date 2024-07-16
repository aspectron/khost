use crate::imports::*;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use std::fs;
use std::io::BufReader;

pub fn load_certs(filename: &str) -> Result<Vec<CertificateDer<'static>>> {
    let certfile = fs::File::open(filename).expect("cannot open certificate file");
    let mut reader = BufReader::new(certfile);
    let certificate = rustls_pemfile::certs(&mut reader)
        .map(|result| result.map_err(Into::into))
        .collect::<Result<Vec<_>>>()?;
    Ok(certificate)
}

pub fn load_private_key(filename: &str) -> Result<PrivateKeyDer<'static>> {
    let keyfile = fs::File::open(filename).expect("cannot open private key file");
    let mut reader = BufReader::new(keyfile);

    loop {
        match rustls_pemfile::read_one(&mut reader)? {
            //.expect("cannot parse private key .pem file") {
            Some(rustls_pemfile::Item::Pkcs1Key(key)) => return Ok(key.into()),
            Some(rustls_pemfile::Item::Pkcs8Key(key)) => return Ok(key.into()),
            Some(rustls_pemfile::Item::Sec1Key(key)) => return Ok(key.into()),
            None => break,
            _ => {}
        }
    }

    Err(Error::custom(format!(
        "no keys found in {:?} (encrypted keys not supported)",
        filename
    )))
}
