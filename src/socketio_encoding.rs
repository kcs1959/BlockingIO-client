//! `rust_socketio`は受け取ったバイト列をLatin-1として解釈してしまう
//!
//! 文字コードを指定することができないし、コードを見たところ英語以外の文字を想定していないようだったので、このモジュールで文字コードを変更する

use encoding::all::{ISO_8859_1, UTF_8};
use encoding::DecoderTrap;
use encoding::EncoderTrap;
use encoding::Encoding;
use rust_socketio::Payload;

/// 受け取った文字列をUTF-8として解釈し直す
///
/// 例:
///  - 「å ¡ãã¦ã¼ã¶ã¼」→「名無しユーザー」
///  - 「ã«ã¼ã 10」→「ルーム10」
pub fn payload_str_to_utf8(payload: &Payload) -> String {
    let bytes = payload_str_to_bytes(payload);
    let converted = UTF_8.decode(&bytes, DecoderTrap::Ignore).unwrap();
    converted
}

pub fn payload_str_to_bytes(payload: &Payload) -> Vec<u8> {
    match payload {
        Payload::String(str) => ISO_8859_1.encode(&str, EncoderTrap::Ignore).unwrap(),
        Payload::Binary(bytes) => bytes.to_vec(),
    }
}

pub trait ToUtf8String {
    fn to_utf8(&self) -> String;
    fn to_utf8_bytes(&self) -> Vec<u8>;
}

impl ToUtf8String for Payload {
    fn to_utf8(&self) -> String {
        payload_str_to_utf8(self)
    }

    fn to_utf8_bytes(&self) -> Vec<u8> {
        payload_str_to_bytes(self)
    }
}
