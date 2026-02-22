//! Event system — add/remove listeners, dispatch with capture/bubble phases.

use super::{Document, NodeId};
use super::node::EventListenerEntry;
use crate::core::volkiwithstds::collections::{String, Vec, HashMap};

/// The phase of event dispatch.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EventPhase {
    None = 0,
    Capturing = 1,
    AtTarget = 2,
    Bubbling = 3,
}

/// An event being dispatched through the DOM tree.
pub struct Event {
    pub event_type: String,
    pub target: NodeId,
    pub current_target: NodeId,
    pub phase: EventPhase,
    pub bubbles: bool,
    pub cancelable: bool,
    pub propagation_stopped: bool,
    pub immediate_propagation_stopped: bool,
    pub default_prevented: bool,
    pub timestamp: u64,
}

impl Event {
    /// Creates a new event.
    pub fn new(event_type: &str, target: NodeId, bubbles: bool, cancelable: bool) -> Self {
        Self {
            event_type: String::from(event_type),
            target,
            current_target: target,
            phase: EventPhase::None,
            bubbles,
            cancelable,
            propagation_stopped: false,
            immediate_propagation_stopped: false,
            default_prevented: false,
            timestamp: 0,
        }
    }

    /// Stops further propagation of the event.
    pub fn stop_propagation(&mut self) {
        self.propagation_stopped = true;
    }

    /// Stops propagation and prevents other listeners on the same target.
    pub fn stop_immediate_propagation(&mut self) {
        self.propagation_stopped = true;
        self.immediate_propagation_stopped = true;
    }

    /// Prevents the default action associated with the event.
    pub fn prevent_default(&mut self) {
        if self.cancelable {
            self.default_prevented = true;
        }
    }
}

/// Options for addEventListener.
pub struct ListenerOptions {
    pub capture: bool,
    pub once: bool,
    pub passive: bool,
}

impl ListenerOptions {
    pub fn new() -> Self {
        Self { capture: false, once: false, passive: false }
    }

    pub fn capture(mut self) -> Self {
        self.capture = true;
        self
    }

    pub fn once(mut self) -> Self {
        self.once = true;
        self
    }

    pub fn passive(mut self) -> Self {
        self.passive = true;
        self
    }
}

/// Registry mapping callback_id → function pointer.
pub struct CallbackRegistry {
    callbacks: HashMap<usize, fn(&mut Event)>,
    next_id: usize,
}

impl CallbackRegistry {
    pub fn new() -> Self {
        Self {
            callbacks: HashMap::new(),
            next_id: 1,
        }
    }

    /// Registers a callback and returns its id.
    pub fn register(&mut self, callback: fn(&mut Event)) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        self.callbacks.insert(id, callback);
        id
    }

    /// Removes a callback.
    pub fn unregister(&mut self, id: usize) {
        self.callbacks.remove(&id);
    }

    /// Invokes a callback by id.
    pub fn invoke(&self, id: usize, event: &mut Event) {
        if let Some(cb) = self.callbacks.get(&id) {
            cb(event);
        }
    }
}

impl Document {
    /// Adds an event listener to a node.
    pub fn add_event_listener(
        &mut self,
        id: NodeId,
        event_type: &str,
        callback_id: usize,
        options: &ListenerOptions,
    ) {
        self.nodes[id.0].event_listeners.push(EventListenerEntry {
            event_type: String::from(event_type),
            callback_id,
            capture: options.capture,
            once: options.once,
            passive: options.passive,
        });
    }

    /// Removes an event listener from a node.
    pub fn remove_event_listener(
        &mut self,
        id: NodeId,
        event_type: &str,
        callback_id: usize,
        capture: bool,
    ) {
        self.nodes[id.0].event_listeners.retain(|l| {
            !(l.event_type.as_str() == event_type && l.callback_id == callback_id && l.capture == capture)
        });
    }

    /// Dispatches an event through the DOM: capture phase → target → bubble phase.
    pub fn dispatch_event(&mut self, event: &mut Event, registry: &CallbackRegistry) {
        // Build the ancestor path from root to target
        let mut path = Vec::new();
        let mut current = self.nodes[event.target.0].parent;
        while let Some(c) = current {
            path.push(c);
            current = self.nodes[c.0].parent;
        }
        // Reverse to get root → target order
        let mut path_reversed = Vec::with_capacity(path.len());
        let mut i = path.len();
        while i > 0 {
            i -= 1;
            path_reversed.push(path[i]);
        }
        let path = path_reversed;

        // Capture phase: root → parent of target
        event.phase = EventPhase::Capturing;
        for &ancestor in path.iter() {
            if event.propagation_stopped {
                break;
            }
            event.current_target = ancestor;
            self.fire_listeners(ancestor, event, registry, true);
        }

        // At-target phase: fire all matching listeners in a single pass
        // (fire_listeners already skips the capture check when phase == AtTarget)
        if !event.propagation_stopped {
            event.phase = EventPhase::AtTarget;
            event.current_target = event.target;
            self.fire_listeners(event.target, event, registry, false);
        }

        // Bubble phase: parent of target → root
        if event.bubbles && !event.propagation_stopped {
            event.phase = EventPhase::Bubbling;
            let mut j = path.len();
            while j > 0 {
                j -= 1;
                if event.propagation_stopped {
                    break;
                }
                event.current_target = path[j];
                self.fire_listeners(path[j], event, registry, false);
            }
        }

        event.phase = EventPhase::None;
    }

    /// Fires matching listeners on a single node.
    fn fire_listeners(
        &mut self,
        node: NodeId,
        event: &mut Event,
        registry: &CallbackRegistry,
        capture_phase: bool,
    ) {
        // Collect matching listener callback_ids to avoid borrow issues
        let mut to_fire = Vec::new();
        let mut to_remove = Vec::new();

        for listener in self.nodes[node.0].event_listeners.iter() {
            if listener.event_type.as_str() != event.event_type.as_str() {
                continue;
            }
            // At target phase: fire both capture and bubble listeners
            let at_target = event.phase == EventPhase::AtTarget;
            if !at_target && listener.capture != capture_phase {
                continue;
            }
            to_fire.push(listener.callback_id);
            if listener.once {
                to_remove.push(listener.callback_id);
            }
        }

        for cb_id in to_fire.iter() {
            if event.immediate_propagation_stopped {
                break;
            }
            registry.invoke(*cb_id, event);
        }

        // Remove once listeners
        if !to_remove.is_empty() {
            self.nodes[node.0].event_listeners.retain(|l| {
                !(l.event_type.as_str() == event.event_type.as_str()
                    && to_remove.contains(&l.callback_id))
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::Document;
    use core::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn test_add_remove_listener() {
        let mut doc = Document::new();
        let div = doc.create_element("div");
        let opts = ListenerOptions::new();
        doc.add_event_listener(div, "click", 1, &opts);
        assert_eq!(doc.nodes[div.0].event_listeners.len(), 1);

        doc.remove_event_listener(div, "click", 1, false);
        assert_eq!(doc.nodes[div.0].event_listeners.len(), 0);
    }

    #[test]
    fn test_dispatch_event() {
        static DISPATCH_CTR: AtomicUsize = AtomicUsize::new(0);
        fn handler(_event: &mut Event) {
            DISPATCH_CTR.fetch_add(1, Ordering::SeqCst);
        }

        let mut doc = Document::new();
        let div = doc.create_element("div");
        doc.append_child(doc.root, div);

        let mut registry = CallbackRegistry::new();
        let cb_id = registry.register(handler);

        let opts = ListenerOptions::new();
        doc.add_event_listener(div, "click", cb_id, &opts);

        let before = DISPATCH_CTR.load(Ordering::SeqCst);
        let mut event = Event::new("click", div, true, false);
        doc.dispatch_event(&mut event, &registry);

        assert!(DISPATCH_CTR.load(Ordering::SeqCst) > before);
    }

    #[test]
    fn test_event_bubbling() {
        static BUBBLE_CTR: AtomicUsize = AtomicUsize::new(0);
        fn handler(_event: &mut Event) {
            BUBBLE_CTR.fetch_add(1, Ordering::SeqCst);
        }

        let mut doc = Document::new();
        let parent = doc.create_element("div");
        let child = doc.create_element("span");
        doc.append_child(doc.root, parent);
        doc.append_child(parent, child);

        let mut registry = CallbackRegistry::new();
        let cb_id = registry.register(handler);

        let opts = ListenerOptions::new();
        doc.add_event_listener(parent, "click", cb_id, &opts);

        let before = BUBBLE_CTR.load(Ordering::SeqCst);
        let mut event = Event::new("click", child, true, false);
        doc.dispatch_event(&mut event, &registry);

        assert!(BUBBLE_CTR.load(Ordering::SeqCst) > before);
    }

    #[test]
    fn test_stop_propagation() {
        static STOP_CTR: AtomicUsize = AtomicUsize::new(0);
        fn stop_handler(event: &mut Event) {
            event.stop_propagation();
        }
        fn count_handler(_event: &mut Event) {
            STOP_CTR.fetch_add(1, Ordering::SeqCst);
        }

        let mut doc = Document::new();
        let parent = doc.create_element("div");
        let child = doc.create_element("span");
        doc.append_child(doc.root, parent);
        doc.append_child(parent, child);

        let mut registry = CallbackRegistry::new();
        let stop_id = registry.register(stop_handler);
        let count_id = registry.register(count_handler);

        let opts = ListenerOptions::new();
        doc.add_event_listener(child, "click", stop_id, &opts);
        doc.add_event_listener(parent, "click", count_id, &opts);

        let before = STOP_CTR.load(Ordering::SeqCst);
        let mut event = Event::new("click", child, true, false);
        doc.dispatch_event(&mut event, &registry);

        assert_eq!(STOP_CTR.load(Ordering::SeqCst), before);
    }

    #[test]
    fn test_once_listener() {
        let mut doc = Document::new();
        let div = doc.create_element("div");
        doc.append_child(doc.root, div);

        let registry = CallbackRegistry::new();

        let opts = ListenerOptions::new().once();
        doc.add_event_listener(div, "click", 99, &opts);
        assert_eq!(doc.nodes[div.0].event_listeners.len(), 1);

        let mut event = Event::new("click", div, true, false);
        doc.dispatch_event(&mut event, &registry);

        // Once listener should be removed after dispatch
        assert_eq!(doc.nodes[div.0].event_listeners.len(), 0);
    }

    #[test]
    fn test_callback_registry() {
        fn noop(_event: &mut Event) {}
        let mut registry = CallbackRegistry::new();
        let id = registry.register(noop);
        assert!(id > 0);
        registry.unregister(id);
    }

    #[test]
    fn test_prevent_default() {
        let mut doc = Document::new();
        let div = doc.create_element("div");
        let mut event = Event::new("click", div, true, true);
        event.prevent_default();
        assert!(event.default_prevented);

        let mut event2 = Event::new("click", div, true, false);
        event2.prevent_default();
        assert!(!event2.default_prevented); // not cancelable
    }
}
