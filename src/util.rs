use roxmltree::Node;

pub fn literal_text_in_node<'a>(node: Node<'a, 'a>) -> &'a str {
    for child in node.children() {
        if child.is_text() {
            return child.text().unwrap_or("");
        }
    }
    ""
}
