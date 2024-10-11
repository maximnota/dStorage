use std::collections::HashMap;

// Define Node struct
#[derive(Debug, Clone)]
struct Node {
    frequency: i64,
    letter: Option<char>, // Some for leaves, None for internal nodes
    left: Option<Box<Node>>,
    right: Option<Box<Node>>,
}

struct Compressor {
    text: String,
}

impl Compressor {
    // Find the smallest item in the list of nodes
    fn find_smallest_item(&self, nodes: &Vec<Node>) -> (usize, Node) {
        let mut smallest_index = 0;
        let mut smallest_node = &nodes[0];

        for (i, node) in nodes.iter().enumerate() {
            if node.frequency < smallest_node.frequency {
                smallest_node = node;
                smallest_index = i;
            }
        }

        (smallest_index, smallest_node.clone())
    }

    // Merge the two smallest nodes and return the new node
    fn merge_smallest_branches(&self, mut nodes: Vec<Node>) -> Node {
        while nodes.len() > 1 {
            let (first_index, first_smallest) = self.find_smallest_item(&nodes);
            nodes.remove(first_index);

            let (second_index, second_smallest) = self.find_smallest_item(&nodes);
            nodes.remove(second_index);

            let merged_node = Node {
                frequency: first_smallest.frequency + second_smallest.frequency,
                letter: None,
                left: Some(Box::new(first_smallest)),
                right: Some(Box::new(second_smallest)),
            };

            nodes.push(merged_node);
        }

        nodes.pop().unwrap() // Return the last node, which is the root of the Huffman tree
    }

    // Generate Huffman codes by traversing the tree
    fn generate_codes(&self, node: &Node, code: String, codes: &mut HashMap<char, String>) {
        if let Some(letter) = node.letter {
            // If it's a leaf node, store the character and its code
            codes.insert(letter, code);
        } else {
            // Recursively traverse the left and right children
            if let Some(ref left) = node.left {
                self.generate_codes(left, format!("{}0", code), codes);
            }
            if let Some(ref right) = node.right {
                self.generate_codes(right, format!("{}1", code), codes);
            }
        }
    }

    // Create a frequency table and compress
    fn compress(&self) {
        let mut nodes: Vec<Node> = Vec::new();
        let mut frequency_map = HashMap::new();

        // Calculate frequencies of characters
        for character in self.text.chars() {
            *frequency_map.entry(character).or_insert(0) += 1;
        }

        // Create nodes from the frequency map
        for (letter, frequency) in frequency_map {
            nodes.push(Node {
                frequency: frequency,
                letter: Some(letter),
                left: None,
                right: None,
            });
        }

        println!("Initial Nodes: {:?}", nodes);

        // Merge branches to create the Huffman tree
        let huffman_tree = self.merge_smallest_branches(nodes);

        // Generate Huffman codes by traversing the tree
        let mut codes = HashMap::new();
        self.generate_codes(&huffman_tree, String::new(), &mut codes);

        println!("Huffman Codes: {:?}", codes);

        // You can now use these codes to encode the text
        let mut encoded_text = String::new();
        for character in self.text.chars() {
            if let Some(code) = codes.get(&character) {
                encoded_text.push_str(code);
            }
        }

        println!("Encoded Text: {}", encoded_text);
    }
}

fn main() {
    let text = String::from("BABABBABBBABABBABABEEEEEZZZIHDIAHIDHIAHD");
    let compressor_struct = Compressor { text };
    compressor_struct.compress();
}

