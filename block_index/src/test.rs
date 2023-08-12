#[cfg(test)]
mod block_index_test {
    use crate::*;

    use std::fs::File;
    use std::io::Read;
    use std::path::PathBuf;
    use tokio::runtime::Runtime;
    use std::future::Future;
    use std::fs;
    use once_cell::sync::Lazy;

    // -1 needed for prevent oferflow in impl
    const NOT_EXIST_INDEX : HeightType = HeightType::MAX - 1;
    // const NOT_EXIST_OFFSET : WeaveOffsetType = WeaveOffsetType::MAX - 1;
    const NOT_EXIST_OFFSET : WeaveOffsetType = -1;
    
    static BLOCK_0_ORIG: Lazy<BlockIndex3JsonEntity> = Lazy::new(|| {
        BlockIndex3JsonEntity {
            tx_root: "P_OiqMNN1s4ltcaq0HXb9VFos_Zz6LFjM8ogUG0vJek".into(),
            weave_size: "0".into(),
            hash: "7wIU7KolICAjClMlcZ38LZzshhI7xGkm2tDCJR7Wvhe3ESUo2-Z4-y0x1uaglRJE".into(),
        }
    });
    static BLOCK_I780_ORIG: Lazy<BlockIndex3JsonEntity> = Lazy::new(|| {
        // height 3527
        BlockIndex3JsonEntity {
            tx_root: "-ooAyZUR49hT5AfXRcoyq0AC5LDyQ3cXuk5koRoPIBY".into(),
            weave_size: "1039029".into(),
            hash: "2-FVrwkpu-fn3y495h2Bhw3MS5JhpYMAQc_8C1ke9whFwB1T-ajdb2Ajk2iWEk0q".into(),
        }
    });
    static BLOCK_4307_ORIG: Lazy<BlockIndex3JsonEntity> = Lazy::new(|| {
        // 4307
        BlockIndex3JsonEntity {
            tx_root: "".into(),
            weave_size: "1039029".into(),
            hash: "0uKcuoohVV_ZV91Cd76PbbT8d8_zCnCGwigRtvJ1hAa6BW0RzqPn4OOSv80WbRFz".into(),
        }
    });
    // static BLOCK_0: Lazy<BlockIndexEntity> = Lazy::new(|| {
        // BLOCK_0_ORIG.decode().unwrap()
    // });
    static BLOCK_I780: Lazy<BlockIndexEntity> = Lazy::new(|| {
        let mut ret = BLOCK_I780_ORIG.decode().unwrap();
        ret.block_size = 439971;
        ret
    });
    static BLOCK_4307: Lazy<BlockIndexEntity> = Lazy::new(|| {
        BLOCK_4307_ORIG.decode().unwrap()
    });

    static INDEX: Lazy<BlockIndex3Json> = Lazy::new(|| {
        let path = "../test_asset/block_index_slice";
        let mut index = BlockIndex3Json::new();
        index.load_sync(&path).unwrap();
        index
    });

    fn run_test<T: Future<Output = Result<(), Box<dyn std::error::Error>>>>(fut: T) -> Result<(), Box<dyn std::error::Error>> {
        let rt = Runtime::new()?;
        rt.block_on(fut)
    }

   #[test]
   fn test_load() -> Result<(), Box<dyn std::error::Error>> {
       let path = "../test_asset/block_index_slice";
       let mut index = BlockIndex3Json::new();
       run_test(index.load(&path))
   }

    #[test]
    fn test_save() -> Result<(), Box<dyn std::error::Error>> {
        let path = "../test_asset/block_index_slice";
        let target_file = "../test_asset/block_index_slice_re";
        let mut index = BlockIndex3Json::new();
        run_test(index.load(&path))?;

        if PathBuf::from(target_file).exists() {
            std::fs::remove_file(target_file)?;
        }

        run_test(index.save(&target_file))?;
        assert!(PathBuf::from(target_file).exists());

        let mut buf1 = Vec::new();
        let mut buf2 = Vec::new();
        File::open(path)?.read_to_end(&mut buf1)?;
        File::open(target_file)?.read_to_end(&mut buf2)?;
        assert_eq!(buf1, buf2);

        std::fs::remove_file(target_file)?;

        Ok(())
    }

   #[test]
   fn test_load_sync() -> Result<(), Box<dyn std::error::Error>> {
       let path = "../test_asset/block_index_slice";
       let mut index = BlockIndex3Json::new();
       index.load_sync(&path)?;
       Ok(())
   }

    #[test]
    fn test_save_sync() -> Result<(), Box<dyn std::error::Error>> {
        let path = "../test_asset/block_index_slice";
        // NOTE. DO NOT USE block_index_slice_re, rust tests will run in parallel and will conflict for shared file
        let target_file = "../test_asset/block_index_slice_re2";
        let mut index = BlockIndex3Json::new();
        run_test(index.load(&path))?;

        if PathBuf::from(target_file).exists() {
            std::fs::remove_file(target_file)?;
        }

        index.save_sync(&target_file)?;
        assert!(PathBuf::from(target_file).exists());

        let mut buf1 = Vec::new();
        let mut buf2 = Vec::new();
        File::open(path)?.read_to_end(&mut buf1)?;
        File::open(target_file)?.read_to_end(&mut buf2)?;
        assert_eq!(buf1, buf2);

        std::fs::remove_file(target_file)?;

        Ok(())
    }


    async fn file_handler(_req: hyper::Request<hyper::Body>) -> Result<hyper::Response<hyper::Body>, hyper::Error> {
        let bytes = fs::read_to_string("../test_asset/block_index_slice").unwrap();
        Ok(hyper::Response::new(hyper::Body::from(bytes)))
    }

    #[test]
    fn test_download() -> Result<(), Box<dyn std::error::Error>> {
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let make_svc = hyper::service::make_service_fn(|_conn| async {
                    Ok::<_, hyper::Error>(hyper::service::service_fn(file_handler))
                });
                let addr = ([127, 0, 0, 1], 1337).into();
                let server = hyper::Server::bind(&addr).serve(make_svc);
                tx.send(addr).unwrap();
                server.await.unwrap();
            });
        });
        let addr = rx.recv().unwrap();

        {
            let peer_url = format!("http://{}", addr);

            let mut index = BlockIndex3Json::new();
            let result = run_test(index.download(&[peer_url]));

            match result {
                Ok(()) => {
                    assert_eq!(index.block_list[4307], *BLOCK_0_ORIG);
                    assert_eq!(index.block_list[0], *BLOCK_4307_ORIG);
                },
                Err(err) => {
                    panic!("Download failed: {:?}", err);
                }
            }
        }
        {
            let peer_url = format!("http://{}", addr);

            let mut index = BlockIndex3Json::new();
            let result = run_test(index.download(&["http://127.0.0.1:1338".into(), peer_url]));

            match result {
                Ok(()) => {
                    assert_eq!(index.block_list[4307], *BLOCK_0_ORIG);
                    assert_eq!(index.block_list[0], *BLOCK_4307_ORIG);
                },
                Err(err) => {
                    panic!("Download failed: {:?}", err);
                }
            }
        }
        {
            let mut index = BlockIndex3Json::new();
            let result = run_test(index.download(&["http://127.0.0.1:1338".into(), "http://127.0.0.1:1339".into()]));

            match result {
                Ok(()) => {
                    panic!("Download should fail");
                },
                Err(_err) => {}
            }
        }

        Ok(())
    }

    #[test]
    fn get_by_height_full() {
        assert!(INDEX.get_by_height_full(NOT_EXIST_INDEX).is_none());
        assert!(INDEX.get_by_height_full(4308).is_none());
        assert_eq!(INDEX.get_by_height_full(4307).unwrap(), *BLOCK_4307);
    }

    #[test]
    fn get_by_height_indep_hash() {
        assert!(INDEX.get_by_height_indep_hash(NOT_EXIST_INDEX).is_none());
        assert!(INDEX.get_by_height_indep_hash(4308).is_none());
        assert_eq!(INDEX.get_by_height_indep_hash(4307).unwrap(), BLOCK_4307.indep_hash);
    }

    #[test]
    fn get_by_height_weave_size() {
        assert!(INDEX.get_by_height_weave_size(NOT_EXIST_INDEX).is_none());
        assert!(INDEX.get_by_height_weave_size(4308).is_none());
        assert_eq!(INDEX.get_by_height_weave_size(4307).unwrap(), BLOCK_4307.weave_size);
    }

    #[test]
    fn get_by_height_tx_root() {
        assert!(INDEX.get_by_height_tx_root(NOT_EXIST_INDEX).is_none());
        assert!(INDEX.get_by_height_tx_root(4308).is_none());
        assert_eq!(INDEX.get_by_height_tx_root(4307), BLOCK_4307.tx_root);
    }

    #[test]
    fn get_by_height_indep_hash_orig() {
        assert!(INDEX.get_by_height_indep_hash_orig(NOT_EXIST_INDEX).is_none());
        assert!(INDEX.get_by_height_indep_hash_orig(4308).is_none());
        assert_eq!(INDEX.get_by_height_indep_hash_orig(4307).unwrap(), BLOCK_4307_ORIG.hash);
    }

    #[test]
    fn get_by_height_weave_size_orig() {
        assert!(INDEX.get_by_height_weave_size_orig(NOT_EXIST_INDEX).is_none());
        assert!(INDEX.get_by_height_weave_size_orig(4308).is_none());
        assert_eq!(INDEX.get_by_height_weave_size_orig(4307).unwrap(), BLOCK_4307_ORIG.weave_size);
    }

    #[test]
    fn get_by_height_tx_root_orig() {
        assert!(INDEX.get_by_height_tx_root_orig(NOT_EXIST_INDEX).is_none());
        assert!(INDEX.get_by_height_tx_root_orig(4308).is_none());
        assert_eq!(INDEX.get_by_height_tx_root_orig(4307).unwrap(), BLOCK_4307_ORIG.tx_root);
    }

    #[test]
    fn _get_block_idx_by_chunk_offset() {
        assert_eq!(INDEX._get_block_idx_by_chunk_offset(NOT_EXIST_OFFSET), None);
        assert_eq!(INDEX._get_block_idx_by_chunk_offset(1039029 + 1), None);

        fn fn_test(idx: usize, block_list: &[BlockIndex3JsonEntity]) -> BlockJsonIdxRet {
            BlockJsonIdxRet {
              idx,
              block_index_json_entity : &block_list[idx]
            }
        }

        assert_eq!(INDEX._get_block_idx_by_chunk_offset(1039029), Some(fn_test(780, &INDEX.block_list)));
        assert_eq!(INDEX._get_block_idx_by_chunk_offset(0), Some(fn_test(4307, &INDEX.block_list)));
        assert_eq!(INDEX._get_block_idx_by_chunk_offset(1), Some(fn_test(4225, &INDEX.block_list)));
        assert_eq!(INDEX._get_block_idx_by_chunk_offset(599058 - 1), Some(fn_test(4225, &INDEX.block_list)));
        assert_eq!(INDEX._get_block_idx_by_chunk_offset(599058), Some(fn_test(4225, &INDEX.block_list)));
        assert_eq!(INDEX._get_block_idx_by_chunk_offset(599058 + 1), Some(fn_test(780, &INDEX.block_list)));
    }

    #[test]
    fn get_by_chunk_offset_full() {
        assert!(INDEX.get_by_chunk_offset_full(NOT_EXIST_OFFSET).is_none());
        assert_eq!(INDEX.get_by_chunk_offset_full(1039029).unwrap(), *BLOCK_I780);
    }

    #[test]
    fn get_by_chunk_offset_indep_hash() {
        assert!(INDEX.get_by_chunk_offset_indep_hash(NOT_EXIST_OFFSET).is_none());
        assert_eq!(INDEX.get_by_chunk_offset_indep_hash(1039029).unwrap(), BLOCK_I780.indep_hash);
    }

    #[test]
    fn get_by_chunk_offset_weave_size() {
        assert!(INDEX.get_by_chunk_offset_weave_size(NOT_EXIST_OFFSET).is_none());
        assert_eq!(INDEX.get_by_chunk_offset_weave_size(1039029).unwrap(), BLOCK_I780.weave_size);
    }

    #[test]
    fn get_by_chunk_offset_tx_root() {
        assert!(INDEX.get_by_chunk_offset_tx_root(NOT_EXIST_OFFSET).is_none());
        assert_eq!(INDEX.get_by_chunk_offset_tx_root(1039029), BLOCK_I780.tx_root);
    }

    #[test]
    fn get_by_chunk_offset_indep_hash_orig() {
        assert!(INDEX.get_by_chunk_offset_indep_hash_orig(NOT_EXIST_OFFSET).is_none());
        assert_eq!(INDEX.get_by_chunk_offset_indep_hash_orig(1039029).unwrap(), BLOCK_I780_ORIG.hash);
    }

    #[test]
    fn get_by_chunk_offset_weave_size_orig() {
        assert!(INDEX.get_by_chunk_offset_weave_size_orig(NOT_EXIST_OFFSET).is_none());
        assert_eq!(INDEX.get_by_chunk_offset_weave_size_orig(1039029).unwrap(), BLOCK_I780_ORIG.weave_size);
    }

    #[test]
    fn get_by_chunk_offset_tx_root_orig() {
        assert!(INDEX.get_by_chunk_offset_tx_root_orig(NOT_EXIST_OFFSET).is_none());
        assert_eq!(INDEX.get_by_chunk_offset_tx_root_orig(1039029).unwrap(), BLOCK_I780_ORIG.tx_root);
    }

}
