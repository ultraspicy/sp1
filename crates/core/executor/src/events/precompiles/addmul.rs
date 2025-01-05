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

    pub a: u32,
    pub b: u32,
    pub c: u32,
    pub d: u32,

    pub result: u32,

    /// The local memory access records.
    pub local_mem_access: Vec<MemoryLocalEvent>,
}
