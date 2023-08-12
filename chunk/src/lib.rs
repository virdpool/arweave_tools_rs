use std::error::Error;
use data_encoding::BASE64URL_NOPAD;
use types::*;
use serde::Deserialize;
use openssl::hash::{Hasher, MessageDigest};

fn sha256(buf: &[u8]) -> Result<Vec<u8>, openssl::error::ErrorStack> {
    let mut hasher = Hasher::new(MessageDigest::sha256())?;
    hasher.update(buf)?;
    Ok(hasher.finish()?.as_ref().to_vec())
}

fn sha256_list(list: &[&[u8]]) -> Result<Vec<u8>, openssl::error::ErrorStack> {
    let mut hasher = Hasher::new(MessageDigest::sha256())?;
    for buf in list {
        hasher.update(buf)?;
    }
    Ok(hasher.finish()?.as_ref().to_vec())
}


pub enum Packing {
    Unpacked,
    Spora25,
    // TODO Spora26
}

impl Packing {
    pub fn from_str(s: &str) -> Result<Packing, Box<dyn Error>> {
        match s {
            "unpacked" => Ok(Packing::Unpacked),
            "spora_2_5" => Ok(Packing::Spora25),
            _ => Err(Box::from(format!("unknown packing {}", s))),
        }
    }
}

pub struct Chunk {
    pub tx_path: ChunkPathType,
    pub data_path: ChunkPathType,
    pub chunk: Vec<u8>,
    pub packing: Packing,
    // TODO packing_addr : Option<Address>
}

#[derive(Deserialize)]
pub struct ChunkJson {
    pub tx_path: String,
    pub data_path: String,
    pub chunk: String,
    pub packing: String,
}

pub fn chunk_from_json(chunk_json: &ChunkJson) -> Result<Chunk, Box<dyn Error>> {
    let chunk_vec = BASE64URL_NOPAD.decode(chunk_json.chunk.as_bytes())?;
    let tx_path = BASE64URL_NOPAD.decode(chunk_json.tx_path.as_bytes())
        .map_err(|e| format!("Failed to decode tx_path: {:?}", e))?;
    let data_path = BASE64URL_NOPAD.decode(chunk_json.data_path.as_bytes())
        .map_err(|e| format!("Failed to decode data_path: {:?}", e))?;
    let packing = Packing::from_str(&chunk_json.packing)?;

    Ok(Chunk {
        tx_path,
        data_path,
        chunk: chunk_vec,
        packing,
    })
}



#[derive(PartialEq, Debug)]
pub struct ValidateTxPathRes {
    pub data_root: ChunkRootType,
    pub tx_start: WeaveOffsetType,
    pub tx_end: WeaveOffsetType,
    pub recall_bucket_offset: WeaveOffsetType,
}

#[derive(PartialEq, Debug)]
pub struct ValidateDataPathRes {
    pub chunk_size: WeaveOffsetType,
    pub offset_diff: WeaveOffsetType,
}


pub fn validate_tx_path(
    tx_path: &ChunkPathType,
    mut chunk_offset: WeaveOffsetType,
    block_index3: &dyn BlockIndex3,
    strict_data_split_threshold: WeaveOffsetType,
) -> Option<ValidateTxPathRes> {
    let block_index_entity = block_index3.get_by_chunk_offset_full(chunk_offset)?;
    if chunk_offset >= strict_data_split_threshold {
        let diff = chunk_offset - strict_data_split_threshold;
        chunk_offset = strict_data_split_threshold + (diff / DATA_CHUNK_SIZE) * DATA_CHUNK_SIZE;
    }
    
    let recall_bucket_offset = chunk_offset - block_index_entity.weave_size;
    let ret = validate_path(block_index_entity.tx_root.unwrap(), recall_bucket_offset, block_index_entity.block_size, tx_path)?;
    
    Some(ValidateTxPathRes {
        data_root: ret.root,
        tx_start: ret.start,
        tx_end: ret.end,
        recall_bucket_offset,
    })
}

pub fn validate_data_path(data_path: &ChunkPathType, val_res: ValidateTxPathRes) -> Option<ValidateDataPathRes> {
    let tx_size = val_res.tx_end - val_res.tx_start;
    let recall_chunk_offset = val_res.recall_bucket_offset - val_res.tx_start;
    let ret = validate_path(val_res.data_root, recall_chunk_offset, tx_size, data_path);
    ret.map(|res| {
        ValidateDataPathRes {
            chunk_size: res.end - res.start,
            offset_diff: res.start - recall_chunk_offset,
        }
    })
}

pub struct ValidateRes {
    root: ChunkRootType,
    start: WeaveOffsetType,
    end: WeaveOffsetType,
}

pub fn validate_path(
    root: ChunkRootType,
    mut offset: WeaveOffsetType,
    block_size: WeaveSizeType,
    any_path: &ChunkPathType,
) -> Option<ValidateRes> {
    if block_size <= 0 {
        return None;
    }
    if offset >= block_size {
        offset = block_size - 1;
    }
    if offset < 0 {
        offset = 0;
    }
    let left: WeaveOffsetType = 0;
    let right = block_size;
    _validate_path_lr(root, offset, left, right, any_path)
}


fn _validate_path_lr(
    tx_root: ChunkRootType,
    offset: WeaveOffsetType,
    left: WeaveOffsetType,
    right: WeaveOffsetType,
    tx_path: &ChunkPathType,
) -> Option<ValidateRes> {
    if tx_path.len() == CHUNKROOT_LENGTH + NOTE_LENGTH {
        let data = &tx_path[0..CHUNKROOT_LENGTH];
        let note = &tx_path[CHUNKROOT_LENGTH..];
        let expd_id = sha256_list(&[&sha256(data).unwrap(), &sha256(note).unwrap()]).unwrap();

        if tx_root != expd_id.as_slice() {
            return None;
        }
        // TEMP SOLUTION
        // Will break when we will hit i128 capacity
        // let note_bn = i128::from_be_bytes(note.try_into().unwrap());
        let note_bn = i128::from_be_bytes((&note[16..]).try_into().unwrap());
        return Some(ValidateRes {
            root: data.try_into().unwrap(),
            start: left,
            end: std::cmp::max(std::cmp::min(right, note_bn), left + 1),
        });
    } else {
        let l = &tx_path[0..CHUNKROOT_LENGTH];
        let r = &tx_path[CHUNKROOT_LENGTH..2 * CHUNKROOT_LENGTH];
        let note = &tx_path[2 * CHUNKROOT_LENGTH..2 * CHUNKROOT_LENGTH + NOTE_LENGTH];
        let rest = &tx_path[2 * CHUNKROOT_LENGTH + NOTE_LENGTH..].to_vec();
        let expd_id = sha256_list(&[&sha256(l).unwrap(), &sha256(r).unwrap(), &sha256(note).unwrap()]).unwrap();

        if tx_root != expd_id.as_slice() {
            return None;
        }

        // TEMP SOLUTION
        // Will break when we will hit i128 capacity
        // let note_bn = i128::from_be_bytes(note.try_into().unwrap());
        let note_bn = i128::from_be_bytes((&note[16..]).try_into().unwrap());
        if offset < note_bn {
            return _validate_path_lr(l.try_into().unwrap(), offset, left, std::cmp::min(right, note_bn), rest);
        } else {
            return _validate_path_lr(r.try_into().unwrap(), offset, std::cmp::max(left, note_bn), right, rest);
        }
    }
}


pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}

#[cfg(test)]
mod test;