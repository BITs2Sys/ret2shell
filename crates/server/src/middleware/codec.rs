//! Add encryption/decryption tower_http layer for all the api requests.

use axum::{
  body::Body,
  http::{Request, header},
  middleware::Next,
  response::Response,
};
use base64::{
  Engine, alphabet,
  engine::{self, GeneralPurpose, GeneralPurposeConfig},
};
use futures::TryStreamExt;
use tokio_util::{
  bytes::{Buf, BufMut, BytesMut},
  codec::{Decoder, Encoder, FramedRead},
  io::StreamReader,
};

use crate::traits::ResponseError;

const B64_TABLE: &str = "SUCaeck4xrsbgtPwnGY56qpm9vWDIZAKVjlf.HFd,E17Tz0iNQ2yJMLh8OoRuX3B";

pub struct Ret2Codec {
  engine: GeneralPurpose,
}

impl Decoder for Ret2Codec {
  type Item = Vec<u8>;
  type Error = std::io::Error;

  fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
    if src.is_empty() || src.len() % 4 != 0 {
      return Ok(None);
    }
    let consumed = src.len();
    let decoded = self
      .engine
      .decode(src.clone())
      .map_err(std::io::Error::other)?;
    src.advance(consumed);
    Ok(Some(decoded))
  }
}

impl Encoder<Vec<u8>> for Ret2Codec {
  type Error = std::io::Error;

  fn encode(&mut self, item: Vec<u8>, dst: &mut BytesMut) -> Result<(), Self::Error> {
    let result = self.engine.encode(&item);
    dst.put(result.as_bytes());
    Ok(())
  }
}

pub async fn encrypt_stream_codec(
  req: Request<Body>, next: Next,
) -> Result<Response<Body>, ResponseError> {
  let headers = req.headers();
  if headers
    .get(header::CONTENT_TYPE)
    .is_some_and(|s| s.eq("application/x-ret2stream"))
  {
    let original_content_type = headers.get("X-Original-Content-Type").map(|s| s.to_str());
    let original_content_type = match original_content_type {
      Some(Ok(s)) => s.to_owned(),
      _ => String::from("application/json"),
    };
    let alphabet = alphabet::Alphabet::new(B64_TABLE).unwrap();
    let config = GeneralPurposeConfig::new()
      .with_decode_allow_trailing_bits(true)
      .with_encode_padding(true)
      .with_decode_padding_mode(engine::DecodePaddingMode::RequireCanonical);
    let engine = GeneralPurpose::new(&alphabet, config);
    let (parts, body) = req.into_parts();
    let body_stream = body.into_data_stream().map_err(std::io::Error::other);
    let new_body =
      FramedRead::new(StreamReader::new(body_stream), Ret2Codec { engine }).into_stream();
    let mut new_req = Request::from_parts(parts, Body::from_stream(new_body));
    new_req
      .headers_mut()
      .insert(header::CONTENT_TYPE, original_content_type.parse().unwrap());
    Ok(next.run(new_req).await)
  } else {
    Ok(next.run(req).await)
  }
}
