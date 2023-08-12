pub type HeightType = u64;

// TODO add _ in middle of name
pub const CHUNKROOT_LENGTH : usize = 32;
pub type ChunkRootType = [u8; CHUNKROOT_LENGTH];

pub const NOTE_LENGTH : usize = 32;
pub type ChunkNoteType = [u8; NOTE_LENGTH];

pub type ChunkPathType = Vec<u8>;

pub const TXROOT_LENGTH : usize = 32;
pub type TxRootType = [u8; TXROOT_LENGTH];

pub const INDEPHASH_LENGTH : usize = 48;
pub type IndepHashType = [u8; INDEPHASH_LENGTH];

// i128 needed for correct chunk verification
pub type WeaveSizeType = i128;
pub type WeaveOffsetType = WeaveSizeType;

pub const DEFAULT_STRICT_DATA_SPLIT_THRESHOLD: WeaveSizeType = 30607159107830;
pub const DATA_CHUNK_SIZE: WeaveSizeType = 256 * 1024;

#[derive(PartialEq, Debug)]
pub struct BlockIndexEntity {
    pub indep_hash: IndepHashType,
    pub weave_size: WeaveSizeType,
    pub tx_root: Option<TxRootType>,
    pub block_size: WeaveSizeType,
}

pub trait BlockIndex {}

pub trait BlockIndex3: BlockIndex {
    fn get_by_height_full(&self, height: HeightType) -> Option<BlockIndexEntity>;
    fn get_by_height_indep_hash(&self, height: HeightType) -> Option<IndepHashType>;
    fn get_by_height_weave_size(&self, height: HeightType) -> Option<WeaveSizeType>;
    fn get_by_height_tx_root(&self, height: HeightType) -> Option<TxRootType>;
    fn get_by_height_indep_hash_orig(&self, height: HeightType) -> Option<String>;
    fn get_by_height_weave_size_orig(&self, height: HeightType) -> Option<String>;
    fn get_by_height_tx_root_orig(&self, height: HeightType) -> Option<String>;

    fn get_by_chunk_offset_full(&self, chunk_offset: WeaveOffsetType) -> Option<BlockIndexEntity>;
    fn get_by_chunk_offset_indep_hash(&self, chunk_offset: WeaveOffsetType) -> Option<IndepHashType>;
    fn get_by_chunk_offset_weave_size(&self, chunk_offset: WeaveOffsetType) -> Option<WeaveSizeType>;
    fn get_by_chunk_offset_tx_root(&self, chunk_offset: WeaveOffsetType) -> Option<TxRootType>;
    fn get_by_chunk_offset_indep_hash_orig(&self, chunk_offset: WeaveOffsetType) -> Option<String>;
    fn get_by_chunk_offset_weave_size_orig(&self, chunk_offset: WeaveOffsetType) -> Option<String>;
    fn get_by_chunk_offset_tx_root_orig(&self, chunk_offset: WeaveOffsetType) -> Option<String>;
}
