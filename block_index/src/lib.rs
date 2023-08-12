use std::error::Error;
use std::fs::File;
use std::io::{Read, Write};
use serde::{Deserialize, Serialize};
use serde_json;
use regex::Regex;
use data_encoding::BASE64URL_NOPAD;
use reqwest;

use types::*;

// TODO move to separate file
////////////////////////////////////////////////////////////////////////////////////////////////////
//  BlockIndex3Json
//  purpose - quick load for pick single value (do not decode all and then throw out)
//  but need validate once loaded
//  maybe I will make BlockIndex3JsonNoValidate later
////////////////////////////////////////////////////////////////////////////////////////////////////


#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct BlockIndex3JsonEntity {
    tx_root: String,
    weave_size: String,
    hash: String,
}

impl BlockIndex3JsonEntity {
    // Lossy decode, (missing block_size)
    fn decode(&self) -> Option<BlockIndexEntity> {
        let weave_size: WeaveSizeType = self.weave_size.parse().ok()?;

        let mut indep_hash: IndepHashType = [0; INDEPHASH_LENGTH];
        BASE64URL_NOPAD.decode_mut(self.hash.as_bytes(), &mut indep_hash).ok()?;

        let tx_root = if self.tx_root.is_empty() {
            None
        } else {
            let mut tx_root_bytes: TxRootType = [0; TXROOT_LENGTH];
            BASE64URL_NOPAD.decode_mut(self.tx_root.as_bytes(), &mut tx_root_bytes).ok()?;
            Some(tx_root_bytes)
        };

        Some(BlockIndexEntity {
            indep_hash,
            weave_size,
            tx_root,
            block_size: 0,
        })
    }
}


fn orig_format3_json_check(json: &[BlockIndex3JsonEntity]) -> Result<(), String> {
    let tx_root_regex = Regex::new(r"(?i)^[-_a-z0-9]{43}$").unwrap();
    let weave_size_regex = Regex::new(r"^\d+$").unwrap();
    let hash_regex = Regex::new(r"(?i)^[-_a-z0-9]{64}$").unwrap();

    if !json.is_empty() {
        for (i, el) in json.iter().enumerate() {
            if el.tx_root != "" && !tx_root_regex.is_match(&el.tx_root) {
                return Err(format!("json[{}].tx_root is not base64url with length 43", i));
            }
            if !weave_size_regex.is_match(&el.weave_size) {
                return Err(format!("json[{}].weave_size is not decimal", i));
            }
            if !hash_regex.is_match(&el.hash) {
                println!("{}", el.hash);
                return Err(format!("json[{}].hash is not base64url with length 64", i));
            }
        }
        let mut prev_weave_size: WeaveSizeType = json[json.len() - 1].weave_size.parse().map_err(|_| format!("Failed to parse weave_size at index {}", json.len() - 1))?;
        for i in (0..json.len() - 1).rev() {
            let weave_size: WeaveSizeType = json[i].weave_size.parse().map_err(|_| format!("Failed to parse weave_size at index {}", i))?;
            if prev_weave_size > weave_size {
                return Err(format!("json[{}] prev_weave_size > weave_size; {} > {}", i, prev_weave_size, weave_size));
            }
            prev_weave_size = weave_size;
        }
    }
    Ok(())

    // ... rest of the code ...
}

#[derive(PartialEq, Debug)]
struct BlockJsonIdxRet<'a> {
    idx: usize,
    block_index_json_entity: &'a BlockIndex3JsonEntity,
}

pub struct BlockIndex3Json {
    block_list: Vec<BlockIndex3JsonEntity>,
    chunk_offset_a: WeaveOffsetType,
    chunk_offset_b: WeaveOffsetType,
}
impl BlockIndex3Json {
    pub fn new() -> Self {
        BlockIndex3Json {
            block_list: Vec::new(),
            chunk_offset_a: 0,
            chunk_offset_b: 0,
        }
    }

    fn _load_from_original_format(&mut self, json: Vec<BlockIndex3JsonEntity>) -> Result<(), Box<dyn Error>> {
        orig_format3_json_check(&json)?;
        self.block_list = json;
        self.chunk_offset_a = self.block_list[self.block_list.len() - 1].weave_size.parse().unwrap();
        self.chunk_offset_b = self.block_list[0].weave_size.parse().unwrap();
        Ok(())
    }

    // WARNING impl is actually not async
    pub async fn save(&self, path: &str) -> Result<(), Box<dyn Error>> {
        let mut file = File::create(path)?;
        let block_list = serde_json::to_string(&self.block_list)?;
        file.write_all(block_list.as_bytes())?;
        Ok(())
    }

    // WARNING impl is actually not async
    pub async fn load(&mut self, path: &str) -> Result<(), Box<dyn Error>> {
        let mut file = File::open(path)?;
        let mut cont = String::new();
        file.read_to_string(&mut cont)?;
        let json: Vec<BlockIndex3JsonEntity> = serde_json::from_str(&cont)?;
        self._load_from_original_format(json)?;
        Ok(())
    }

    pub fn save_sync(&self, path: &str) -> Result<(), Box<dyn Error>> {
        let mut file = File::create(path)?;
        let block_list = serde_json::to_string(&self.block_list)?;
        file.write_all(block_list.as_bytes())?;
        Ok(())
    }

    pub fn load_sync(&mut self, path: &str) -> Result<(), Box<dyn Error>> {
        let mut file = File::open(path)?;
        let mut cont = String::new();
        file.read_to_string(&mut cont)?;
        let json: Vec<BlockIndex3JsonEntity> = serde_json::from_str(&cont)?;
        self._load_from_original_format(json)?;
        Ok(())
    }

    fn _get_block_idx_by_chunk_offset(&self, chunk_offset: WeaveOffsetType) -> Option<BlockJsonIdxRet> {
        if self.chunk_offset_a > chunk_offset || self.chunk_offset_b < chunk_offset {
            return None;
        }

        let last_idx = self.block_list.len() - 1;
        let mut idx_a = last_idx;
        let mut idx_b = 0;
        let mut idx_c = (idx_b + idx_a) / 2;
        let mut co_c = self.block_list[idx_c].weave_size.parse::<WeaveSizeType>().unwrap();

        let mut ret_block_idx;
        loop {
            if co_c == chunk_offset {
                ret_block_idx = idx_c - 1;
                break;
            }
            if idx_c == idx_b {
                ret_block_idx = idx_b;
                break;
            }

            if co_c > chunk_offset {
                idx_b = idx_c;
                idx_c = (idx_b + idx_a) / 2;
            } else {
                idx_a = idx_c;
                idx_c = (idx_b + idx_a) / 2;
            }

            co_c = self.block_list[idx_c].weave_size.parse::<WeaveSizeType>().unwrap();
        }

        let mut ret = &self.block_list[ret_block_idx];
        while ret_block_idx < last_idx {
            let probe_block = &self.block_list[ret_block_idx + 1];
            if ret.weave_size != probe_block.weave_size {
                break;
            }
            ret_block_idx += 1;
            ret = probe_block;
        }

        Some(BlockJsonIdxRet{
          idx : ret_block_idx,
          block_index_json_entity : ret
      })
    }

    pub async fn download(&mut self, peer_url_list: &[String]) -> Result<(), Box<dyn Error>> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()?;

        let mut last_err: Box<dyn Error> = "No valid peer URL found".into();
        for peer_url in peer_url_list {
            let url = format!("{}/block_index", peer_url);
            println!("{}", url);
            match client.get(&url).send().await {
                Ok(response) => {
                    let bytes = response.bytes().await?;
                    let json_str = String::from_utf8(bytes.to_vec())?;
                    let json: Vec<BlockIndex3JsonEntity> = serde_json::from_str(&json_str)?;
                    self._load_from_original_format(json)?;
                    return Ok(());
                }
                Err(err) => {
                    last_err = err.into();
                }
            }
        }

        Err(last_err)
    }
}

impl BlockIndex for BlockIndex3Json {}
impl BlockIndex3 for BlockIndex3Json {
    fn get_by_height_full(&self, height: HeightType) -> Option<BlockIndexEntity> {
        let idx = self.block_list.len().checked_sub(height as usize + 1)?;

        let block_index_json_entity = self.block_list.get(idx)?;
        let prev_block_index_entity = self.block_list.get(idx + 1);

        let weave_size: WeaveSizeType = block_index_json_entity.weave_size.parse().ok()?;
        let prev_weave_size: WeaveSizeType = prev_block_index_entity
            .map(|prev| prev.weave_size.parse().ok())
            .unwrap_or(Some(0))?;

        let mut indep_hash: IndepHashType = [0; INDEPHASH_LENGTH];
        BASE64URL_NOPAD.decode_mut(block_index_json_entity.hash.as_bytes(), &mut indep_hash).ok()?;

        let tx_root = if block_index_json_entity.tx_root.is_empty() {
            None
        } else {
            let mut tx_root_bytes: TxRootType = [0; TXROOT_LENGTH];
            BASE64URL_NOPAD.decode_mut(block_index_json_entity.tx_root.as_bytes(), &mut tx_root_bytes).ok()?;
            Some(tx_root_bytes)
        };

        Some(BlockIndexEntity {
            indep_hash,
            weave_size,
            tx_root,
            block_size: weave_size - prev_weave_size,
        })
    }

    fn get_by_height_indep_hash(&self, height: HeightType) -> Option<IndepHashType> {
        let idx = self.block_list.len().checked_sub(height as usize + 1)?;
        let block_index_json_entity = self.block_list.get(idx)?;
        let mut indep_hash: IndepHashType = [0; INDEPHASH_LENGTH];
        BASE64URL_NOPAD.decode_mut(block_index_json_entity.hash.as_bytes(), &mut indep_hash).ok()?;
        Some(indep_hash)
    }

    fn get_by_height_weave_size(&self, height: HeightType) -> Option<WeaveSizeType> {
        let idx = self.block_list.len().checked_sub(height as usize + 1)?;
        let block_index_json_entity = self.block_list.get(idx)?;
        Some(block_index_json_entity.weave_size.parse().ok()?)
    }

    fn get_by_height_tx_root(&self, height: HeightType) -> Option<TxRootType> {
        let idx = self.block_list.len().checked_sub(height as usize + 1)?;
        let block_index_json_entity = self.block_list.get(idx)?;
        if block_index_json_entity.tx_root.is_empty() {
            None
        } else {
            let mut tx_root_bytes: TxRootType = [0; TXROOT_LENGTH];
            BASE64URL_NOPAD.decode_mut(block_index_json_entity.tx_root.as_bytes(), &mut tx_root_bytes).ok()?;
            Some(tx_root_bytes)
        }
    }

    fn get_by_height_indep_hash_orig(&self, height: HeightType) -> Option<String> {
        let idx = self.block_list.len().checked_sub(height as usize + 1)?;
        Some(self.block_list.get(idx)?.hash.clone())
    }

    fn get_by_height_weave_size_orig(&self, height: HeightType) -> Option<String> {
        let idx = self.block_list.len().checked_sub(height as usize + 1)?;
        Some(self.block_list.get(idx)?.weave_size.clone())
    }

    fn get_by_height_tx_root_orig(&self, height: HeightType) -> Option<String> {
        let idx = self.block_list.len().checked_sub(height as usize + 1)?;
        Some(self.block_list.get(idx)?.tx_root.clone())
    }

    fn get_by_chunk_offset_full(&self, chunk_offset: WeaveOffsetType) -> Option<BlockIndexEntity> {
        let BlockJsonIdxRet{idx, block_index_json_entity} = self._get_block_idx_by_chunk_offset(chunk_offset)?;

        let prev_block_index_entity = self.block_list.get(idx + 1);
        let weave_size = block_index_json_entity.weave_size.parse::<WeaveSizeType>().unwrap();
        let prev_weave_size = prev_block_index_entity
            .map(|prev| prev.weave_size.parse::<WeaveSizeType>().unwrap())
            .unwrap_or(0);

        let mut indep_hash: IndepHashType = [0; INDEPHASH_LENGTH];
        let mut tx_root: TxRootType = [0; TXROOT_LENGTH];
        BASE64URL_NOPAD.decode_mut(block_index_json_entity.hash.as_bytes(), &mut indep_hash).ok()?;
        BASE64URL_NOPAD.decode_mut(block_index_json_entity.tx_root.as_bytes(), &mut tx_root).ok()?;

        Some(BlockIndexEntity {
            indep_hash,
            weave_size,
            tx_root : Some(tx_root),
            block_size: weave_size - prev_weave_size,
        })
    }

    fn get_by_chunk_offset_indep_hash(&self, chunk_offset: WeaveOffsetType) -> Option<IndepHashType> {
        let BlockJsonIdxRet{idx:_idx, block_index_json_entity} = self._get_block_idx_by_chunk_offset(chunk_offset)?;
        let mut indep_hash: IndepHashType = [0; INDEPHASH_LENGTH];
        BASE64URL_NOPAD.decode_mut(block_index_json_entity.hash.as_bytes(), &mut indep_hash).ok()?;
        Some(indep_hash)
    }

    fn get_by_chunk_offset_weave_size(&self, chunk_offset: WeaveOffsetType) -> Option<WeaveSizeType> {
        let BlockJsonIdxRet{idx:_idx, block_index_json_entity} = self._get_block_idx_by_chunk_offset(chunk_offset)?;
        Some(block_index_json_entity.weave_size.parse::<WeaveSizeType>().unwrap())
    }

    fn get_by_chunk_offset_tx_root(&self, chunk_offset: WeaveOffsetType) -> Option<TxRootType> {
        let BlockJsonIdxRet{idx:_idx, block_index_json_entity} = self._get_block_idx_by_chunk_offset(chunk_offset)?;
        let mut tx_root: TxRootType = [0; TXROOT_LENGTH];
        BASE64URL_NOPAD.decode_mut(block_index_json_entity.tx_root.as_bytes(), &mut tx_root).ok()?;
        Some(tx_root)
    }

    fn get_by_chunk_offset_indep_hash_orig(&self, chunk_offset: WeaveOffsetType) -> Option<String> {
        let BlockJsonIdxRet{idx:_idx, block_index_json_entity} = self._get_block_idx_by_chunk_offset(chunk_offset)?;
        Some(block_index_json_entity.hash.clone())
    }

    fn get_by_chunk_offset_weave_size_orig(&self, chunk_offset: WeaveOffsetType) -> Option<String> {
        let BlockJsonIdxRet{idx:_idx, block_index_json_entity} = self._get_block_idx_by_chunk_offset(chunk_offset)?;
        Some(block_index_json_entity.weave_size.clone())
    }

    fn get_by_chunk_offset_tx_root_orig(&self, chunk_offset: WeaveOffsetType) -> Option<String> {
        let BlockJsonIdxRet{idx:_idx, block_index_json_entity} = self._get_block_idx_by_chunk_offset(chunk_offset)?;
        Some(block_index_json_entity.tx_root.clone())
    }
}

#[cfg(test)]
mod test;
