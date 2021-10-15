// Copyright 2018 Fredrik Portström <https://portstrom.com>
// This is free software distributed under the terms specified in
// the file LICENSE at the top-level directory of this distribution.

pub fn parse_supplementary(
    context: &mut ::Context,
    heading_node: &::Node,
    nodes: &[::Node],
    output: &mut bool,
) -> usize {
    if *output {
        ::add_warning(context, heading_node, ::WarningMessage::Duplicate);
    }
    *output = true;
    let mut node_index = 0;
    while let Some(node) = nodes.get(node_index) {
        if let ::Node::Heading { .. } = node {
            break;
        }
        node_index += 1;
        ::add_warning(context, node, ::WarningMessage::Supplementary);
    }
    node_index
}
