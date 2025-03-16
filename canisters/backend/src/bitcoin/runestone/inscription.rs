use crate::bitcoin_lib::{constants::MAX_SCRIPT_ELEMENT_SIZE, opcodes, script};
use ordinals::Rune;
use tag::Tag;

pub mod tag;

const PROTOCOL_ID: [u8; 3] = *b"ord";
const BODY_TAG: [u8; 0] = [];

#[derive(Default)]
pub struct Inscription {
    pub body: Option<Vec<u8>>,
    pub content_encoding: Option<Vec<u8>>,
    pub content_type: Option<Vec<u8>>,
    pub delegate: Option<Vec<u8>>,
    pub duplicate_field: bool,
    pub incomplete_field: bool,
    pub metadata: Option<Vec<u8>>,
    pub metaprotocol: Option<Vec<u8>>,
    pub parents: Vec<Vec<u8>>,
    pub pointer: Option<Vec<u8>>,
    pub rune: Option<Vec<u8>>,
    pub unrecognized_even_field: bool,
}

impl Inscription {
    pub fn new(body: Option<Vec<u8>>, content_type: Option<Vec<u8>>, rune: Rune) -> Self {
        Self {
            body,
            content_type,
            rune: Some(rune.commitment()),
            ..Default::default()
        }
    }

    pub fn append_reveal_script_to_builder(&self, mut builder: script::Builder) -> script::Builder {
        builder = builder
            .push_opcode(opcodes::OP_FALSE)
            .push_opcode(opcodes::all::OP_IF)
            .push_slice(PROTOCOL_ID);

        Tag::ContentType.append(&mut builder, &self.content_type);
        Tag::ContentEncoding.append(&mut builder, &self.content_encoding);
        Tag::Metaprotocol.append(&mut builder, &self.metaprotocol);
        Tag::Parent.append_array(&mut builder, &self.parents);
        Tag::Delegate.append(&mut builder, &self.delegate);
        Tag::Pointer.append(&mut builder, &self.pointer);
        Tag::Metadata.append(&mut builder, &self.metadata);
        Tag::Rune.append(&mut builder, &self.rune);
        if let Some(body) = &self.body {
            builder = builder.push_slice(BODY_TAG);
            for chunk in body.chunks(MAX_SCRIPT_ELEMENT_SIZE) {
                builder = builder.push_slice::<&script::PushBytes>(chunk.try_into().unwrap());
            }
        }

        builder.push_opcode(opcodes::all::OP_ENDIF)
    }
}
