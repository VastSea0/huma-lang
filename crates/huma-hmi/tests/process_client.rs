use huma_hmi::{HmiError, HmiValue, ProcessClient, ProtocolVersion};
use std::path::Path;
use std::time::Duration;

fn helper() -> &'static Path {
    Path::new(env!("CARGO_BIN_EXE_hmi-test-module"))
}

#[test]
fn ayri_surec_baslatir_cagirir_ve_kapatir() {
    let mut client = ProcessClient::spawn(
        helper(),
        "test",
        ProtocolVersion::V1_0,
        Duration::from_secs(2),
    )
    .expect("HMI test süreci başlamalı");
    assert_eq!(
        client
            .call("topla", vec![HmiValue::Number(2.0), HmiValue::Number(3.0)])
            .unwrap(),
        HmiValue::Number(5.0)
    );
    client.shutdown().expect("HMI süreci temiz kapanmalı");
}

#[test]
fn yanlis_yanit_kimligi_fail_closed_davranir() {
    let mut client = ProcessClient::spawn(
        helper(),
        "test",
        ProtocolVersion::V1_0,
        Duration::from_secs(2),
    )
    .unwrap();
    assert!(matches!(
        client.call("yanlis_id", Vec::new()),
        Err(HmiError::ProtocolViolation(_))
    ));
}

#[test]
fn zaman_asimi_cocuk_sureci_sonlandirir() {
    let mut client = ProcessClient::spawn(
        helper(),
        "test",
        ProtocolVersion::V1_0,
        Duration::from_secs(2),
    )
    .unwrap();
    client.set_timeout(Duration::from_millis(50)).unwrap();
    assert!(matches!(
        client.call("bekle", Vec::new()),
        Err(HmiError::Timeout(_))
    ));
}
