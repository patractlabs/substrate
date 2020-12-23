use sp_std::collections::btree_map::BTreeMap;
use add_getters_setters::AddSetter;
use crate::trace_runtime::NestedRuntimeWrapper;

#[derive(Debug, Default, AddSetter)]
pub struct Record {
    #[set]
    runtime: Option<NestedRuntimeWrapper>,
    host_func_count: BTreeMap<String, u8>,
}

impl Record {
    pub fn is_init(&self) -> bool {
        if self.runtime.is_some() {
            true
        } else {
            false
        }
    }

    pub fn count(&mut self, host_func: &str) {
        if let Some(count) = self.host_func_count.get_mut(&host_func.to_string()) {
            *count = count.checked_add(1).unwrap();
        } else {
            self.host_func_count.insert(host_func.to_string(), 1);
        }
    }
}

impl Drop for Record {
    fn drop(&mut self) {
        println! {"{:#?}\n", self};
    }
}

environmental::environmental!(record: Record);

pub fn set_and_run_with_record<F, R>(record: &mut Record, f: F) -> R
    where
        F: FnOnce() -> R,
{
    record::using(record, f)
}

pub fn with_record<F: FnOnce(&mut Record) -> R, R>(f: F) -> Option<R> {
    record::with(f)
}