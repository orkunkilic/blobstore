use std::iter::Peekable;

use bitcoin::{
    opcodes::{
        all::{OP_ENDIF, OP_IF, OP_PUSHBYTES_0, OP_PUSHBYTES_32},
        OP_0,
    },
    script::{Instruction, Instructions},
};

pub fn find_pattern_instructions(
    instructions: &mut Peekable<Instructions>,
) -> Option<(Vec<u8>, Vec<u8>)> {
    /*
        OP_FALSE
        OP_IF
        OP_PUSH "blob"
        OP_1
        32 [sha256 hash of the blob]
        OP_0
        4 [size of the blob in bytes]
        OP_ENDIF
    */
    while let Some(instruction) = instructions.next().transpose().ok()? {
        match instruction {
            Instruction::Op(OP_PUSHBYTES_0) => (),
            _ => continue,
        }

        let instruction = instructions.next().transpose().ok()??;
        if instruction != Instruction::Op(OP_IF) {
            continue;
        }

        if let Instruction::PushBytes(blob) = instructions.next().transpose().ok()?? {
            // Check if blob is "blob"
            if blob.as_bytes() != b"blob" {
                continue;
            }
        } else {
            continue;
        };

        let instruction = instructions.next().transpose().ok()??;
        if instruction != Instruction::Op(OP_PUSHBYTES_32) {
            continue;
        }

        let hash = if let Instruction::PushBytes(hash) = instructions.next().transpose().ok()?? {
            hash
        } else {
            continue;
        };

        let instruction = instructions.next().transpose().ok()??;
        if instruction != Instruction::Op(OP_0) {
            continue;
        }

        let size = if let Instruction::PushBytes(size) = instructions.next().transpose().ok()?? {
            size
        } else {
            continue;
        };

        let instruction = instructions.next().transpose().ok()??;
        if instruction != Instruction::Op(OP_ENDIF) {
            continue;
        }

        return Some((hash.as_bytes().to_vec(), size.as_bytes().to_vec()));
    }

    None
}
