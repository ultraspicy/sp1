use serde::{Deserialize, Serialize};

use crate::events::{
    memory::{MemoryReadRecord, MemoryWriteRecord},
    LookupId, MemoryLocalEvent,
};

/// AddMulEvent.
///
/// This event is emitted when a a*b + c*d operation is performed.
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct AddMulEvent {
    /// Lookup identifier
    pub lookup_id: LookupId,
    /// Shard number
    pub shard: u32,
    /// Clock cycle
    pub clk: u32,

    /// a value as list of words
    pub a: u32,
    /// Memory records for a
    pub a_memory_records: Vec<MemoryReadRecord>,
    /// b value as list of words
    pub b: u32,
    /// Memory records for b
    pub b_memory_records: Vec<MemoryReadRecord>,
    /// c value as list of words
    pub c: u32,
    /// Memory records for c
    pub c_memory_records: Vec<MemoryReadRecord>,
    /// d value as list of words
    pub d: u32,
    /// Memory records for d
    pub d_memory_records: Vec<MemoryReadRecord>,

    /// Memory records for result (written back to a_ptr)
    pub result_memory_records: Vec<MemoryWriteRecord>,

    /// Local memory access events
    pub local_mem_access: Vec<MemoryLocalEvent>,
}
