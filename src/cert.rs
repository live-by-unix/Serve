use crate::log;
use dirs::home_dir;
use rcgen::generate_simple_self_signed;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use rustls_pemfile::{certs, pkcs8_private_keys};
use std::fs::{create_dir_all, File};
use std::io::{BufReader, Write};

pub fn load() -> (Vec<CertificateDer<'static>>, PrivateKeyDer<'static>) {
    let dir = home_dir().unwrap().join(".serve");

    create_dir_all(&dir).unwrap();

    let cert_path = dir.join("cert.pem");
    let key_path = dir.join("key.pem");

    if !cert_path.exists() || !key_path.exists() {
        let cert = generate_simple_self_signed(vec![
            "localhost".into(),
            "127.0.0.1".into(),
        ])
        .unwrap();

        let cert_pem = cert.cert.pem();
        let key_pem = cert.key_pair.serialize_pem();

        File::create(&cert_path)
            .unwrap()
            .write_all(cert_pem.as_bytes())
            .unwrap();

        File::create(&key_path)
            .unwrap()
            .write_all(key_pem.as_bytes())
            .unwrap();

        log::cert(&dir.display().to_string());
    }

    let mut cert_reader =
        BufReader::new(File::open(cert_path).unwrap());

    let mut key_reader =
        BufReader::new(File::open(key_path).unwrap());

    let certs = certs(&mut cert_reader)
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    let key = pkcs8_private_keys(&mut key_reader)
        .next()
        .unwrap()
        .unwrap();

    (certs, key.into())
}
