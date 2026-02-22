//! Mutation observer — records DOM changes for interested observers.

use super::NodeId;
use crate::core::volkiwithstds::collections::{String, Vec};

/// The kind of mutation that was recorded.
pub enum MutationRecord {
    /// Children were added or removed.
    ChildList {
        target: NodeId,
    },
    /// An attribute was changed.
    Attributes {
        target: NodeId,
        attribute_name: String,
        old_value: Option<String>,
    },
    /// Text content of a text node was changed.
    CharacterData {
        target: NodeId,
        old_value: String,
    },
}

impl MutationRecord {
    fn clone_record(&self) -> Self {
        match self {
            MutationRecord::ChildList { target } => MutationRecord::ChildList { target: *target },
            MutationRecord::Attributes { target, attribute_name, old_value } => {
                MutationRecord::Attributes {
                    target: *target,
                    attribute_name: String::from(attribute_name.as_str()),
                    old_value: match old_value {
                        Some(v) => Some(String::from(v.as_str())),
                        None => None,
                    },
                }
            }
            MutationRecord::CharacterData { target, old_value } => {
                MutationRecord::CharacterData {
                    target: *target,
                    old_value: String::from(old_value.as_str()),
                }
            }
        }
    }
}

/// Options controlling what an observer watches.
pub struct MutationObserverOptions {
    pub child_list: bool,
    pub attributes: bool,
    pub character_data: bool,
    pub subtree: bool,
    pub attribute_old_value: bool,
    pub character_data_old_value: bool,
}

impl MutationObserverOptions {
    pub fn new() -> Self {
        Self {
            child_list: false,
            attributes: false,
            character_data: false,
            subtree: false,
            attribute_old_value: false,
            character_data_old_value: false,
        }
    }

    pub fn child_list(mut self) -> Self {
        self.child_list = true;
        self
    }

    pub fn attributes(mut self) -> Self {
        self.attributes = true;
        self
    }

    pub fn character_data(mut self) -> Self {
        self.character_data = true;
        self
    }

    pub fn subtree(mut self) -> Self {
        self.subtree = true;
        self
    }
}

/// A mutation observer watches a target node for changes.
pub struct MutationObserver {
    pub target: NodeId,
    pub options: MutationObserverOptions,
    pub records: Vec<MutationRecord>,
    pub callback_id: usize,
}

impl MutationObserver {
    pub fn new(target: NodeId, options: MutationObserverOptions, callback_id: usize) -> Self {
        Self {
            target,
            options,
            records: Vec::new(),
            callback_id,
        }
    }

    /// Returns and clears all pending mutation records.
    pub fn take_records(&mut self) -> Vec<MutationRecord> {
        let mut taken = Vec::new();
        core::mem::swap(&mut taken, &mut self.records);
        taken
    }
}

use super::Document;

#[allow(dead_code)]
impl Document {
    /// Registers a mutation observer on a target node.
    pub fn observe(
        &mut self,
        target: NodeId,
        options: MutationObserverOptions,
        callback_id: usize,
    ) -> usize {
        let idx = self.mutation_observers.len();
        self.mutation_observers.push(MutationObserver::new(target, options, callback_id));
        idx
    }

    /// Removes a mutation observer by index.
    pub fn disconnect_observer(&mut self, index: usize) {
        if index < self.mutation_observers.len() {
            self.mutation_observers.remove(index);
        }
    }

    /// Returns pending records for an observer (and clears them).
    pub fn take_observer_records(&mut self, index: usize) -> Vec<MutationRecord> {
        if index < self.mutation_observers.len() {
            self.mutation_observers[index].take_records()
        } else {
            Vec::new()
        }
    }

    /// Records a ChildList mutation. Called by tree manipulation methods.
    pub(crate) fn record_mutation(&mut self, record: MutationRecord) {
        if self.mutation_observers.is_empty() {
            return;
        }

        let target = match &record {
            MutationRecord::ChildList { target } => *target,
            MutationRecord::Attributes { target, .. } => *target,
            MutationRecord::CharacterData { target, .. } => *target,
        };

        // Find observers interested in this mutation
        let mut observer_indices = Vec::new();
        for (i, obs) in self.mutation_observers.iter().enumerate() {
            let watches = obs.target == target
                || (obs.options.subtree && self.is_descendant_of(target, obs.target));

            if !watches {
                continue;
            }

            let interested = match &record {
                MutationRecord::ChildList { .. } => obs.options.child_list,
                MutationRecord::Attributes { .. } => obs.options.attributes,
                MutationRecord::CharacterData { .. } => obs.options.character_data,
            };

            if interested {
                observer_indices.push(i);
            }
        }

        // Push a clone to all matching observers
        let last_idx = if observer_indices.is_empty() { 0 } else { observer_indices.len() - 1 };
        for (pos, &idx) in observer_indices.iter().enumerate() {
            if pos == last_idx {
                // Move the original into the last observer to avoid an extra clone
                self.mutation_observers[idx].records.push(record);
                return;
            }
            self.mutation_observers[idx].records.push(record.clone_record());
        }
    }

    /// Checks if `node` is a descendant of `ancestor`.
    fn is_descendant_of(&self, node: NodeId, ancestor: NodeId) -> bool {
        let mut current = self.nodes[node.0].parent;
        while let Some(p) = current {
            if p == ancestor {
                return true;
            }
            current = self.nodes[p.0].parent;
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::Document;

    #[test]
    fn test_observer_creation() {
        let mut doc = Document::new();
        let div = doc.create_element("div");
        let opts = MutationObserverOptions::new().child_list();
        let idx = doc.observe(div, opts, 1);
        assert_eq!(idx, 0);
    }

    #[test]
    fn test_observer_take_records() {
        let mut doc = Document::new();
        let div = doc.create_element("div");
        let opts = MutationObserverOptions::new().child_list();
        let idx = doc.observe(div, opts, 1);

        // Manually record a mutation
        doc.record_mutation(MutationRecord::ChildList { target: div });

        let records = doc.take_observer_records(idx);
        assert_eq!(records.len(), 1);

        // Should be empty now
        let records2 = doc.take_observer_records(idx);
        assert_eq!(records2.len(), 0);
    }

    #[test]
    fn test_disconnect_observer() {
        let mut doc = Document::new();
        let div = doc.create_element("div");
        let opts = MutationObserverOptions::new().child_list();
        let idx = doc.observe(div, opts, 1);
        doc.disconnect_observer(idx);
        assert_eq!(doc.mutation_observers.len(), 0);
    }
}
