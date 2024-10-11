use std::io; // Uncomment this for I/O operations

// Define node struct
#[derive(Debug, Clone)]
struct Node {
    frequency: i64,
    letter: Option<char>,
}

struct Compressor {
    text: String,
}

impl Compressor {
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

    fn merge_smallest_branches(&self, mut nodes: Vec<Node>) -> Vec<Node> {
        while nodes.len() > 1 {
            let (first_index, first_smallest) = self.find_smallest_item(&nodes);
            nodes.remove(first_index); // Remove the first smallest

            let (second_index, second_smallest) = self.find_smallest_item(&nodes);
            nodes.remove(second_index); // Remove the second smallest

            let merged_node = Node {
                frequency: first_smallest.frequency + second_smallest.frequency,
                letter: None, // Internal nodes have no character
            };

            nodes.push(merged_node);
        }

        nodes
    }

    fn compress(&self) {
        let mut nodes: Vec<Node> = Vec::new();
        let mut frequency_map = std::collections::HashMap::new();

        for character in self.text.chars() {
            *frequency_map.entry(character).or_insert(0) += 1;
        }

        // Create nodes from the frequency map
        for (letter, frequency) in frequency_map {
            if Some(letter) {
                nodes.push(Node {
                    frequency: frequency,
                    letter: Some(letter),
                });

            } if None {
                println!("No letters unfortunately")
            }
        }

        println!("Initial Nodes: {:?}", nodes);

        let compressed_tree = self.merge_smallest_branches(nodes);

        println!("Final Huffman Tree: {:?}", compressed_tree);
    }
}

fn main() {
    let text = String::from("AABBBBDDDDDDCCC");
    let compressor_struct = Compressor {
        text: text,
    };

    compressor_struct.compress();
}

