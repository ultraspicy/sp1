use serde::{Deserialize, Serialize};
use crate::events::{
    memory::{MemoryReadRecord, MemoryWriteRecord},
    LookupId, MemoryLocalEvent,
};

/// AddMul Event.
///
/// basically it is a inner product
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct AddMulEvent {
    /// Lookup identifier
    pub lookup_id: LookupId,
    /// Shard number
    pub shard: u32,
    /// Clock cycle
    pub clk: u32,

    /// p
    pub p: u32,
    /// q
    pub q: u32,
    /// r
    pub r: u32,
    /// s
    pub s: u32,

    /// p_ptr - 
    pub p_ptr: u32,
    /// q_ptr
    pub q_ptr: u32,
    /// r_ptr
    pub r_ptr: u32,
    /// s_ptr
    pub s_ptr: u32,

    /// p_memory_records
    pub p_memory_records: MemoryWriteRecord,
    /// q_memory_records
    pub q_memory_records: MemoryReadRecord,
    /// r_memory_records
    pub r_memory_records: MemoryReadRecord,
    /// s_memory_records
    pub s_memory_records: MemoryReadRecord,

    /// local_mem_access
    pub local_mem_access: Vec<MemoryLocalEvent>,
}

