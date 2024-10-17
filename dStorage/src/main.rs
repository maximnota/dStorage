use std::collections::HashMap;
use std::fs;
use std::io;
use std::net::{TcpListener, SocketAddr, TcpStream};
use std::thread;
use std::io::prelude::*;

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
    fn compress(&self) -> (String, HashMap<char, String>) {
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

        //println!("Initial Nodes: {:?}", nodes);

        // Merge branches to create the Huffman tree
        let huffman_tree = self.merge_smallest_branches(nodes);

        // Generate Huffman codes by traversing the tree
        let mut codes = HashMap::new();
        self.generate_codes(&huffman_tree, String::new(), &mut codes);

        //println!("Huffman Codes: {:?}", codes);

        // You can now use these codes to encode the text
        let mut encoded_text = String::new();
        for character in self.text.chars() {
            if let Some(code) = codes.get(&character) {
                encoded_text.push_str(code);
            }
        }

        //println!("Encoded Text: {}", encoded_text);
        (encoded_text, codes) // Return the encoded text and the encoding table
    }
}

// Decoder struct
struct Decoder {
    encoded_text: String,
    encoding_table: HashMap<char, String>,
}

impl Decoder {
    fn decode(&self) -> String {
        let mut binary:String = String::new();
        let mut decoded_text:String = String::new();
        let reversed_table: HashMap<String, char> = self.encoding_table
            .iter()
            .map(|(k, v)| (v.clone(), *k)) // Reverse the encoding table
            .collect();

        for bit in self.encoded_text.chars() {
            binary.push(bit);
            if let Some(&letter) = reversed_table.get(&binary) {
                decoded_text.push(letter);
                binary.clear();
            }
        }

        //println!("Decoded text: {}", decoded_text);
        decoded_text
    }
}


struct Slicer {
    text: String,
    slice_amount: usize, 
}

impl Slicer {
    fn slice(&self) -> Vec<&str> {
        let text_len = self.text.len();
        let mut slices = vec![];

        if self.slice_amount == 0 {
            return slices; 
        }

        let whole_slices = text_len % self.slice_amount == 0;
        let slice_len = text_len / self.slice_amount;
        
        if whole_slices {
            for i in 0..self.slice_amount {
                let start = i * slice_len;
                let end = start + slice_len;
                slices.push(&self.text[start..end]);
            }
        } else {
            for i in 0..(self.slice_amount - 1) {
                let start = i * slice_len;
                let end = start + slice_len;
                slices.push(&self.text[start..end]);
            }
            let start = (self.slice_amount - 1) * slice_len;
            slices.push(&self.text[start..]);
        }
        
        slices
    }
}

struct Compiler<'a> {
    slices: Vec<&'a str>,
}

impl<'a> Compiler<'a> {
    fn compile(&self) -> String {
        let mut result = String::new();
        for slice in &self.slices {
            result.push_str(slice);
        }
        result
    }
}

struct Receiver {
    primary_port: i32,
    secondary_port: i32,
    response: Option<fn(TcpStream)>,
}

impl Receiver {
    fn receive(&self) {
        let addrs = [
            SocketAddr::from(([127, 0, 0, 1], self.primary_port as u16)),
            SocketAddr::from(([127, 0, 0, 1], self.secondary_port as u16)),
        ];

        // Try binding the listener to both ports
        let listener = TcpListener::bind(&addrs[..]).expect("Failed to bind to address");

        println!("Server listening on ports {} and {}", self.primary_port, self.secondary_port);

        // Accept incoming connections
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let response = self.response;

                    thread::spawn(move || {
                        match response {
                            Some(func) => {
                                func(stream);
                            }
                            None => {
                                handle_client(stream, None);
                            }
                        }
                    });
                }
                Err(e) => {
                    eprintln!("Failed to accept connection: {}", e);
                }
            }
        }
    }
}

fn handle_client(mut stream: TcpStream, response: Option<String>) {
    let mut buffer = [0; 1024];
    let bytes_read = stream.read(&mut buffer).expect("Failed to read from client!");

    let request = String::from_utf8_lossy(&buffer[..bytes_read]);
    println!("Received request: {}", request);

    match response {
        Some(response) => {
            stream.write(response.as_bytes()).expect("Failed to write response.");
        }
        None => {
            println!("No response to write.");
        }
    }
}



struct Request {
    ip: SocketAddr,
    message: String,
}

impl Request {
    fn sendRequest(&self) {
        let mut stream = TcpStream::connect(self.ip)
                       .expect("Couldn't connect to the server...");
        stream.write(self.message.as_bytes()).expect("Failed to write message");
    }
}



fn upload(file_path:&str) {
    let text = fs::read_to_string(file_path).expect("Unable to read text within file");

    let slicer = Slicer {
        text:text.clone(),
        slice_amount: 3,
    };
    let slices = slicer.slice();

    let compiler = Compiler {
        slices
    };
    let result = compiler.compile();
    println!("Original text {}", result);

    //for slice in slices.iter() {
    //    let compressor_struct = Compressor { text: text.clone() };
    //    let (encoded_text, encoding_table) = compressor_struct.compress();
    //    println!("{}", encoded_text);
    //}

    // Compress the text
    
    // Create a decoder and decode the text
    // let decoder_struct = Decoder {
    //    encoded_text,
    //    encoding_table,
    //};
    //decoder_struct.decode();

}

fn clean_file_path(input: &str) -> String {
    input.replace("\\", "")  // Remove all backslashes
}


fn main() {
    //loop {
    //    let mut choice = String::new();
    //    let mut file_path = String::new();
    //    println!("Would you like to upload or download a file? Enter 1 for upload, 2 for download, or 0 to quit:");
    //    io::stdin().read_line(&mut choice).expect("Sorry, unable to read your input");
    //    let choice = choice.trim();  // Trim the newline characters
    //    println!("Your choice was {choice}");
    //    if choice == "1" {
    //        //Upload
    //        println!("Choose a file to upload - enter the path to that file:");
    //        io::stdin().read_line(&mut file_path).expect("Sorry, unable to read your input");
    //        let file_path = file_path.trim();  // Trim the newline characters
    //        let clean_file_path = clean_file_path(file_path);
    //        println!("You selected the file: {file_path} to upload.");
    //        upload(&clean_file_path);
    //    } else if choice == "2" {
    //        println!("Choose a file to download:");
    //        io::stdin().read_line(&mut file_path).expect("Sorry, unable to read your input");
    //        let file_path = file_path.trim();  // Trim the newline characters
    //        clean_file_path(file_path);
    //        println!("You selected the file: {file_path} to download.");
    //    } else if choice == "0" {
    //        println!("Quitting the app.");
    //        break;
    //    } else {
    //        println!("Invalid choice, sorry!");
    //    }
    //}
    //let reciever = Receiver {
    //    primary_port: 6942,
    //    secondary_port: 5439,
    //    response: None,
    //};
    //reciever.receive();
    //println!("Test");
}
