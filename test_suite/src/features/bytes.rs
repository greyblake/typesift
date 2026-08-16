//! The `bytes` feature: both types are leaves.

use bytes::{Bytes, BytesMut};
use typesift::TypeSift;

use crate::helpers::{assert_same_refs, check_consistency};

#[derive(Debug, TypeSift)]
struct Frame {
    payload: Bytes,
    scratch: BytesMut,
    trailers: Vec<Bytes>,
    name: String,
}

fn frame() -> Frame {
    Frame {
        payload: Bytes::from_static(b"payload"),
        scratch: BytesMut::from(&b"scratch"[..]),
        trailers: vec![Bytes::from_static(b"a"), Bytes::from_static(b"b")],
        name: "frame".to_string(),
    }
}

#[test]
fn buffers_are_found_wherever_they_are_nested() {
    let frame = frame();
    assert_same_refs(
        &frame.sift::<Bytes>(),
        &[&frame.payload, &frame.trailers[0], &frame.trailers[1]],
    );
    assert_same_refs(&frame.sift::<BytesMut>(), &[&frame.scratch]);
    assert_eq!(frame.sift::<String>(), ["frame"]);
}

#[test]
fn buffers_are_leaves() {
    let frame = frame();
    // The bytes a buffer holds are not searched.
    assert!(frame.sift::<u8>().is_empty());
    assert!(frame.sift::<Vec<u8>>().is_empty());
}

#[test]
fn entry_points_agree_for_bytes() {
    check_consistency::<_, Bytes>(&frame());
    check_consistency::<_, BytesMut>(&frame());
    check_consistency::<_, Frame>(&frame());
}
