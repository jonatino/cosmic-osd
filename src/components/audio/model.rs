// Copyright 2026 System76 <info@system76.com>
// SPDX-License-Identifier: GPL-3.0-only

use std::collections::HashMap;

use cosmic_settings_audio_client::{self as audio_client};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct NodeVolume {
    pub volume: u32,
    pub mute: bool,
}

#[derive(Debug, Default)]
pub struct Model {
    sinks: Nodes,
    sources: Nodes,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
struct NodeId(u32);

#[derive(Debug, Default)]
struct Node {
    volume: u32,
    mute: bool,
}

impl Node {
    fn value(&self) -> NodeVolume {
        NodeVolume {
            volume: self.volume,
            mute: self.mute,
        }
    }
}

#[derive(Debug, Default)]
struct Nodes {
    active: Option<NodeId>,
    nodes: HashMap<NodeId, Node>,
}

impl Nodes {
    fn active(&self) -> Option<NodeVolume> {
        Some(self.nodes.get(&self.active?)?.value())
    }

    pub fn remove(&mut self, node_id: NodeId) -> bool {
        self.active.take_if(|active| *active == node_id);
        self.nodes.remove(&node_id).is_some()
    }
}

pub enum Response {
    SinkVolume(NodeVolume),
    SourceVolume(NodeVolume),
}

impl Model {
    pub fn update(&mut self, event: audio_client::Event) -> Option<Response> {
        match event {
            audio_client::Event::NodeMute(node_id, mute) => {
                let node_id = NodeId(node_id);
                if let Some(node) = self.sinks.nodes.get_mut(&node_id) {
                    let old_value = node.value();
                    node.mute = mute;
                    let value = node.value();
                    if self.sinks.active == Some(node_id) && old_value != value {
                        return Some(Response::SinkVolume(value));
                    }
                } else if let Some(node) = self.sources.nodes.get_mut(&node_id) {
                    let old_value = node.value();
                    node.mute = mute;
                    let value = node.value();
                    if self.sources.active == Some(node_id) && old_value != value {
                        return Some(Response::SourceVolume(value));
                    }
                }
            }

            audio_client::Event::NodeVolume(node_id, volume, _balance) => {
                let node_id = NodeId(node_id);
                if let Some(node) = self.sinks.nodes.get_mut(&node_id) {
                    let old_value = node.value();
                    node.volume = volume;
                    let value = node.value();
                    if self.sinks.active == Some(node_id) && old_value != value {
                        return Some(Response::SinkVolume(value));
                    }
                } else if let Some(node) = self.sources.nodes.get_mut(&node_id) {
                    let old_value = node.value();
                    node.volume = volume;
                    let value = node.value();
                    if self.sources.active == Some(node_id) && old_value != value {
                        return Some(Response::SourceVolume(value));
                    }
                }
            }

            audio_client::Event::DefaultSink(node_id) => {
                let node_id = NodeId(node_id);
                self.sinks.active = Some(node_id);
                if let Some(value) = self.sinks.active() {
                    return Some(Response::SinkVolume(value));
                }
            }

            audio_client::Event::DefaultSource(node_id) => {
                let node_id = NodeId(node_id);
                self.sources.active = Some(node_id);
                if let Some(value) = self.sources.active() {
                    return Some(Response::SourceVolume(value));
                }
            }

            audio_client::Event::Node(node_id, node) => {
                let node_id = NodeId(node_id);
                if node.is_sink {
                    self.sinks
                        .nodes
                        .entry(node_id)
                        .or_insert_with(Node::default);
                } else {
                    self.sources
                        .nodes
                        .entry(node_id)
                        .or_insert_with(Node::default);
                }
            }

            audio_client::Event::RemoveNode(node_id) => {
                let node_id = NodeId(node_id);
                if !self.sinks.remove(node_id) {
                    self.sources.remove(node_id);
                }
            }

            _ => (),
        }

        None
    }
}
