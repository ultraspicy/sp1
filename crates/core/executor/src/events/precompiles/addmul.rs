use serde::{Deserialize, Serialize};
use crate::events::{
    memory::{MemoryReadRecord, MemoryWriteRecord},
    LookupId, MemoryLocalEvent,
};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct AddMulEvent {
    /// Lookup identifier
    pub lookup_id: LookupId,
    /// Shard number
    pub shard: u32,
    /// Clock cycle
    pub clk: u32,

    pub p: u32,
    pub q: u32,
    pub r: u32,
    pub s: u32,

    pub local_mem_access: Vec<MemoryLocalEvent>,
}