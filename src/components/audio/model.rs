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
    pub active_sink: NodeVolume,
    pub active_source: NodeVolume,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
struct NodeId(u32);

#[derive(Debug, Default)]
struct Node {
    volume: u32,
    mute: bool,
}

#[derive(Debug, Default)]
struct Nodes {
    active: Option<NodeId>,
    nodes: HashMap<NodeId, Node>,
}

impl Nodes {
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
                    node.mute = mute;
                    if self.sinks.active == Some(node_id) && self.active_sink.mute != mute {
                        self.active_sink.mute = mute;
                        return Some(Response::SinkVolume(self.active_source));
                    }
                } else if let Some(node) = self.sources.nodes.get_mut(&node_id) {
                    node.mute = mute;
                    if self.sources.active == Some(node_id) && self.active_source.mute != mute {
                        self.active_source.mute = mute;
                        return Some(Response::SourceVolume(self.active_source));
                    }
                }
            }

            audio_client::Event::NodeVolume(node_id, volume, _balance) => {
                let node_id = NodeId(node_id);
                if let Some(node) = self.sinks.nodes.get_mut(&node_id) {
                    node.volume = volume;
                    if self.sinks.active == Some(node_id) {
                        let changed = self.active_sink.mute != node.mute
                            || self.active_sink.volume != node.volume;
                        self.active_sink.mute = node.mute;
                        self.active_sink.volume = node.volume;

                        return changed.then_some(Response::SinkVolume(self.active_sink));
                    }
                } else if let Some(node) = self.sources.nodes.get_mut(&node_id) {
                    node.volume = volume;
                    if self.sources.active == Some(node_id) {
                        let changed = self.active_source.mute != node.mute
                            || self.active_source.volume != node.volume;
                        self.active_source.mute = node.mute;
                        self.active_source.volume = node.volume;
                        return changed.then_some(Response::SourceVolume(self.active_source));
                    }
                }
            }

            audio_client::Event::DefaultSink(node_id) => {
                let node_id = NodeId(node_id);
                self.sinks.active = Some(node_id);
                if let Some(node) = self.sinks.nodes.get(&node_id) {
                    self.active_sink.mute = node.mute;
                    self.active_sink.volume = node.volume;
                    return Some(Response::SinkVolume(self.active_sink));
                }
            }

            audio_client::Event::DefaultSource(node_id) => {
                let node_id = NodeId(node_id);
                self.sources.active = Some(node_id);
                if let Some(node) = self.sources.nodes.get(&node_id) {
                    self.active_source.mute = node.mute;
                    self.active_source.volume = node.volume;
                    return Some(Response::SourceVolume(self.active_source));
                }
            }

            audio_client::Event::Node(node_id, node) => {
                let node_id = NodeId(node_id);
                if node.is_sink {
                    let node = self
                        .sinks
                        .nodes
                        .entry(node_id)
                        .or_insert_with(Node::default);

                    if self.sinks.active == Some(node_id) {
                        self.active_sink.mute = node.mute;
                        self.active_sink.volume = node.volume;
                    }
                } else {
                    let node = self
                        .sources
                        .nodes
                        .entry(node_id)
                        .or_insert_with(Node::default);

                    if self.sources.active == Some(node_id) {
                        self.active_source.mute = node.mute;
                        self.active_source.volume = node.volume;
                    }
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
