use std::ops::Range;
use std::path::PathBuf;

use crate::table::sstable;
use crate::*;

// Options are params for creating Engine object.
//
// This package provides DefaultOptions which contains options that should
// work for most applications. Consider using that as a starting point before
// customizing it for your own needs.
pub struct Options {
    	// 1. Mandatory flags
	// -------------------
	// Directory to store the data in. Should exist and be writable.
	pub dir: PathBuf,

    // base_size is th maximum L1 size before trigger a compaction.
	// The L2 size is 10x of the base size, L3 size is 100x of the base size.
    pub base_size: u64,

    // Maximum number of tables to keep in memory, before stalling.
    pub num_mem_tables: usize,

    pub max_block_cache_size: i64,

    // Number of compaction workers to run concurrently.
    pub num_compactors: usize,

    pub table_builder_options: sstable::TableBuilderOptions,

    pub remote_compaction_addr: String,

    // instance_id is used to compose SST file name.
    pub instance_id: u32,

    pub cfs: [CFConfig; NUM_CFS],

    pub recovery_concurrency: usize,

    pub preparation_concurrency: usize,

    // Max mem size is dynamically adjusted for each time the mem-table get flushed.
	// The formula is (factor * write_bytes_per_second)
	// And limited in range [2MB, 256MB].
	pub max_mem_table_size_factor: usize,
}

impl Default for Options {
    fn default() -> Self {
        Self { 
            dir: Default::default(), 
            base_size: 64 << 20, 
            num_mem_tables: 16, 
            max_block_cache_size: 0,
            num_compactors: 3,
            table_builder_options: Default::default(), 
            remote_compaction_addr: Default::default(), 
            instance_id: Default::default(), 
            cfs: [CFConfig::new(true, 3), CFConfig::new(false, 2), CFConfig::new(true, 1)],
            recovery_concurrency: Default::default(), 
            preparation_concurrency: Default::default(),
            max_mem_table_size_factor: 256,
        }
    }
}

#[derive(Default, Clone, Copy)]
pub struct CFConfig {
    pub managed: bool,
    pub max_levels: usize,
}

impl CFConfig {
    pub fn new(managed: bool, max_levels: usize) -> Self {
        Self {
            managed,
            max_levels,
        }
    }
}

pub trait IDAllocator: Sync + Send {
    // alloc_id returns the last id, and last_id - count is valid.
    fn alloc_id(&self, count: usize) -> std::result::Result<Vec<u64>, String>;
}

// MetaChangeListener is used to notify the engine user that engine meta has changed.
pub trait MetaChangeListener: Sync + Send {
    fn on_change(&self, e: kvenginepb::ChangeSet);
}

pub trait RecoverHandler {
    // Recovers from the shard's state to the state that is stored in the toState property.
	// So the Engine has a chance to execute pre-split command.
	// If toState is nil, the implementation should recovers to the latest state.
    fn recover(&self, engine: &Engine, shard: &Shard, info: &ShardMeta) -> Result<()>;
}

pub trait MetaIterator {
    fn iterate<F>(&self, f: F) -> Result<()> where F: FnMut(kvenginepb::ChangeSet);
}
