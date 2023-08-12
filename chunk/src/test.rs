#[cfg(test)]
mod chunk_test {
    use crate::*;
    // use types::*;
    use block_index::*;
    use once_cell::sync::Lazy;
    use std::fs;
    use data_encoding::BASE64URL_NOPAD;

    static INDEX: Lazy<BlockIndex3Json> = Lazy::new(|| {
        let path = "../test_asset/block_index_slice";
        let mut index = BlockIndex3Json::new();
        index.load_sync(&path).unwrap();
        index
    });

    static CHUNK1_OFFSET: WeaveOffsetType = 1;
    static CHUNK2_OFFSET: WeaveOffsetType = 599059;
    static CHUNK1_JSON: Lazy<String> = Lazy::new(|| {
        fs::read_to_string(format!("../test_asset/chunk_{}.json", CHUNK1_OFFSET)).unwrap()
    });
    static CHUNK2_JSON: Lazy<String> = Lazy::new(|| {
        fs::read_to_string(format!("../test_asset/chunk_{}.json", CHUNK2_OFFSET)).unwrap()
    });

    #[test]
    fn test_chunk_from_json() {
        let chunk1_json: ChunkJson = serde_json::from_str(&*CHUNK1_JSON).unwrap();
        let chunk2_json: ChunkJson = serde_json::from_str(&*CHUNK2_JSON).unwrap();

        let _chunk1_unpacked = chunk_from_json(&chunk1_json).unwrap();
        let _chunk2_unpacked = chunk_from_json(&chunk2_json).unwrap();

    }

    #[test]
    fn test_validate_tx_path() {
        let chunk1_json: ChunkJson = serde_json::from_str(&*CHUNK1_JSON).unwrap();
        let chunk2_json: ChunkJson = serde_json::from_str(&*CHUNK2_JSON).unwrap();
        
        let chunk1_unpacked = chunk_from_json(&chunk1_json).unwrap();
        let chunk2_unpacked = chunk_from_json(&chunk2_json).unwrap();

        let tx_val_res_chunk1 = validate_tx_path(&chunk1_unpacked.tx_path, CHUNK1_OFFSET, &*INDEX, DEFAULT_STRICT_DATA_SPLIT_THRESHOLD)
            .expect("!tx_val_res");

        let data_root_chunk1: [u8; 32] = BASE64URL_NOPAD.decode("kuMLOSJKG7O4NmSBY9KZ2PjU-5O4UBNFl_-kF9FnW7w".as_bytes())
            .unwrap()
            .try_into()
            .expect("Wrong length for data_root");
        assert_eq!(tx_val_res_chunk1, ValidateTxPathRes {
            data_root: data_root_chunk1,
            tx_start: 0,
            tx_end: 599058,
            recall_bucket_offset: -599057, // it's ok for block 0
        });

        let tx_val_res_chunk2 = validate_tx_path(&chunk2_unpacked.tx_path, CHUNK2_OFFSET, &*INDEX, DEFAULT_STRICT_DATA_SPLIT_THRESHOLD)
            .expect("!tx_val_res");

        let data_root_chunk2: [u8; 32] = BASE64URL_NOPAD.decode("nyGPB30FMq2Bx7TRNXInl6rKFSN4W5na9RycpGbT5IA".as_bytes())
            .unwrap()
            .try_into()
            .expect("Wrong length for data_root");
        assert_eq!(tx_val_res_chunk2, ValidateTxPathRes {
            data_root: data_root_chunk2,
            tx_start: 0,
            tx_end: 439971,
            recall_bucket_offset: -439970, // it's NOT ok for block N
        });
    }

    #[test]
    fn test_validate_data_path() {
        // Retrieving chunk1 and chunk2 from previous example
        let chunk1_json: ChunkJson = serde_json::from_str(&*CHUNK1_JSON).unwrap();
        let chunk2_json: ChunkJson = serde_json::from_str(&*CHUNK2_JSON).unwrap();
        
        let chunk1_unpacked = chunk_from_json(&chunk1_json).unwrap();
        let chunk2_unpacked = chunk_from_json(&chunk2_json).unwrap();

        let tx_val_res_chunk1 = validate_tx_path(&chunk1_unpacked.tx_path, CHUNK1_OFFSET, &*INDEX, DEFAULT_STRICT_DATA_SPLIT_THRESHOLD)
            .expect("!tx_val_res");
        let data_val_res_chunk1 = validate_data_path(&chunk1_unpacked.data_path, tx_val_res_chunk1)
            .expect("!data_val_res");
        assert_eq!(data_val_res_chunk1, ValidateDataPathRes {
            chunk_size: 262144,
            offset_diff: 599057,
        });

        let tx_val_res_chunk2 = validate_tx_path(&chunk2_unpacked.tx_path, CHUNK2_OFFSET, &*INDEX, DEFAULT_STRICT_DATA_SPLIT_THRESHOLD)
            .expect("!tx_val_res");
        let data_val_res_chunk2 = validate_data_path(&chunk2_unpacked.data_path, tx_val_res_chunk2)
            .expect("!data_val_res");
        println!("{:?}", data_val_res_chunk2);
        assert_eq!(data_val_res_chunk2, ValidateDataPathRes {
            chunk_size: 262144,
            offset_diff: 439970,
        });
    }

}