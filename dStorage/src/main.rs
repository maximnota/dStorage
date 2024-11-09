use std::collections::HashMap;
use std::fs;
use std::io;
use std::net::{TcpListener, SocketAddr, TcpStream};
use std::thread;
use std::io::prelude::*;
use rusqlite::{Connection, Result, params};
use std::fs::File;
use std::path::Path;
use std::fs::OpenOptions;
use std::io::{Write, Read};


struct FilePointer {
    id: i32,
    ip: String,
    file_name: String,
    dictionary_in_place: bool,
    encoded_text_in_place: bool,
}

impl FilePointer {
    fn write_dictionary(&self, dictionary_bytes: Vec<u8>) -> Result<(), io::Error> {
        let directory = format!("{}", self.ip);
        let file_name = format!("{}/{}_dictionary.txt", directory, self.file_name);
        let path = Path::new(&file_name);

        if !Path::new(&directory).exists() {
            fs::create_dir_all(&directory)?;
        }

        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)             
            .open(path)?;

        file.write_all(&dictionary_bytes)?;
        println!("Dictionary written to: {}", file_name);
        Ok(())
    }

    fn write_encoded_text(&self, encoded_text_bytes: Vec<u8>) -> Result<(), io::Error> {
        let directory = format!("{}", self.ip);
        let file_name = format!("{}/{}_encoded_text.txt", directory, self.file_name);
        let path = Path::new(&file_name);

        if !Path::new(&directory).exists() {
            fs::create_dir_all(&directory)?;
        }

        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)            
            .open(path)?;

        file.write_all(&encoded_text_bytes)?;
        println!("Encoded text written to: {}", file_name);
        Ok(())
    }

    fn read_encoded_text(&self) -> Result<Vec<u8>, io::Error> {
        let file_name = format!("{}/{}_encoded_text.txt", self.ip, self.file_name);
        let path = Path::new(&file_name);

        if path.exists() {
            println!("Reading encoded text from: {}", file_name);
            let data = fs::read(path)?;
            Ok(data)
        } else {
            println!("File doesn't exist: {}", file_name);
            Err(io::Error::new(io::ErrorKind::NotFound, "File not found"))
        }
    }
}

// Define Node struct
#[derive(Debug, Clone)]
struct Node {
    frequency: i64,
    letter: Option<char>,    
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
fn bits_to_u8(bits: &[u8]) -> Result<u8, String> {
    if bits.len() != 8 {
        return Err("The input must contain exactly 8 bits".into());
    }

    let mut result = 0u8;
    for (i, &bit) in bits.iter().enumerate() {
        if bit != 0 && bit != 1 {
            return Err("Invalid bit value".into());
        }
        result |= bit << (7 - i); // Shift the bit to its correct position
    }
    Ok(result)
}


fn handle_file_upload_request(mut stream: TcpStream, file_name: String) -> Result<(), Box<dyn std::error::Error>> {
    let mut buffer = [0; 1024];
    let bytes_read = stream.read(&mut buffer)?;

    let conn = Connection::open("pointers.db")?;

    let table_name = "file_pointers";
    let mut stmt = conn.prepare("SELECT name FROM sqlite_master WHERE type='table' AND name=?1")?;
    let table_exists = stmt.exists([table_name])?;

    if !table_exists {
        conn.execute(
            "CREATE TABLE file_pointers (
                id INTEGER PRIMARY KEY,
                ip TEXT NOT NULL,
                fileName TEXT NOT NULL,
                dictionaryInPlace TEXT NOT NULL,
                encodedTextInPlace TEXT NOT NULL
            )",
            [],
        )?;
    }

    let local_addr = stream.local_addr().unwrap();
    let ip = local_addr.ip().to_string();

    if buffer[0] == 0b1 {
        println!("Request with dictionary");
        let dictionary_buffer = &buffer[1..bytes_read];

        let mut stmt = conn.prepare("SELECT id, ip, fileName, dictionaryInPlace, encodedTextInPlace FROM file_pointers WHERE dictionaryInPlace='FALSE' AND ip=?1")?;
        let result = stmt.query_map([ip], |row| {
            Ok(FilePointer {
                id: row.get(0)?,
                ip: row.get(1)?,
                file_name: row.get(2)?,
                dictionary_in_place: row.get(3)?,
                encoded_text_in_place: row.get(4)?,
            })
        })?;

        for file_pointer in result {
            let mut file_pointer = file_pointer?;
            file_pointer.write_dictionary(dictionary_buffer.to_vec())?;
        }

    } else if buffer[0] == 0b0 {
        println!("Request with encoded data");
        let encoded_text_buffer = &buffer[1..bytes_read];

        let mut stmt = conn.prepare("SELECT id, ip, fileName, dictionaryInPlace, encodedTextInPlace FROM file_pointers WHERE encodedTextInPlace='FALSE' AND ip=?1")?;
        let result = stmt.query_map([ip], |row| {
            Ok(FilePointer {
                id: row.get(0)?,
                ip: row.get(1)?,
                file_name: row.get(2)?,
                dictionary_in_place: row.get(3)?,
                encoded_text_in_place: row.get(4)?,
            })
        })?;

        for file_pointer in result {
            let mut file_pointer = file_pointer?;
            file_pointer.write_encoded_text(encoded_text_buffer.to_vec())?;
        }
    }

    Ok(())
}

fn handle_file_download(mut stream: TcpStream, file_name: String) -> Result<(), Box<dyn std::error::Error>> {
    let mut buffer = [0; 1024];
    let bytes_read = stream.read(&mut buffer)?;

    let conn = Connection::open("pointers.db")?;
    let table_name = "file_pointers";
    
    let mut stmt = conn.prepare("SELECT name FROM sqlite_master WHERE type='table' AND name=?1")?;
    let table_exists = stmt.exists([table_name])?;

    let local_addr = stream.local_addr()?;
    let ip = local_addr.ip().to_string();
    if !table_exists {
        let response = format!("Declined: never had file '{}' uploaded to the machine from IP {}", file_name, ip);
        stream.write_all(response.as_bytes())?;
        return Ok(());
    } else {
        let mut stmt = conn.prepare(
            "SELECT id, ip, fileName, dictionaryInPlace, encodedTextInPlace FROM file_pointers WHERE fileName=?1 AND ip=?2"
        )?;
        
        let mut rows = stmt.query_map(params![file_name, ip], |row| {
            Ok(FilePointer {
                id: row.get(0)?,
                ip: row.get(1)?,
                file_name: row.get(2)?,
                dictionary_in_place: row.get(3)?,
                encoded_text_in_place: row.get(4)?,
            })
        })?;

        if let Some(file_pointer) = rows.next() {
            let file_pointer = file_pointer?;             
            if file_pointer.encoded_text_in_place {
                let encoded_data = file_pointer.read_encoded_text()?;
                stream.write_all(&encoded_data)?;
                println!("Sent encoded data for file: {}", file_pointer.file_name);
            } else {
                let response = "Encoded text not available yet";
                stream.write_all(response.as_bytes())?;
            }
        } else {
            let response = format!("Declined: No file '{}' found for IP {}", file_name, ip);
            stream.write_all(response.as_bytes())?;
        }
    }

    Ok(())
}
// Function to send a decline response
fn send_decline_response(addr: std::net::SocketAddr) {
    let send_request = Request {
        ip: addr,
        message: "Declined: not part of network".to_string(),
    };
    send_request.sendRequest();
}

fn handle_requests(mut stream: TcpStream, network_id: String) -> Result<(), Box<dyn std::error::Error>> {
    let mut buffer = [0; 1024];
    let bytes_read = stream.read(&mut buffer)?;

    let current_stage_code = u32::from_be_bytes(buffer[..4].try_into()?);
    let conn = Connection::open("pointers.db")?;
    let table_name = format!("connections{}", network_id);

    let table_exists = conn.prepare(&format!(
        "SELECT name FROM sqlite_master WHERE type='table' AND name='{}'", table_name
    ))?.exists([])?;

    if !table_exists {
        conn.execute(
            &format!("CREATE TABLE {} (ip TEXT NOT NULL, stage TEXT NOT NULL)", table_name),
            [],
        )?;
    }

    let ip = stream.peer_addr()?.ip().to_string();
    let user_exists = conn.prepare(&format!(
        "SELECT ip FROM {} WHERE ip = ?1", table_name
    ))?.exists(params![ip])?;

    if user_exists {
        let mut stmt = conn.prepare(&format!("SELECT stage FROM {} WHERE ip = ?1", table_name))?;
        let stage: String = stmt.query_row(params![ip], |row| row.get(0))?;
        match stage.as_str() {
            "0011" => {
                println!("Upload request stage");

            },
            "0001" => {
                println!("Download starting");
                let file_name = String::from_utf8_lossy(&buffer[..4]).to_string();
                handle_file_download(stream, file_name);
            },
            "0111" => {
                println!("Finishing upload..");
                conn.execute(&format!("UPDATE {} SET stage = {} WHERE ip = {}", table_name, "0000".to_string(), ip), [])?;

            }
            _ => println!("Unknown stage"),
        }
    } else {
        match current_stage_code {
            0b1111 => println!("Join request"),
            0b0111 => println!("Leave request"),
            0b0011 => {
                println!("Upload request");
                if !user_exists {
                    conn.execute(&format!("INSERT INTO {} values ({}, {})", table_name, ip, "0011".to_string()), [])?;
                } else {
                    conn.execute(&format!("UPDATE {} SET stage = {} WHERE ip = {}", table_name, "0011".to_string(), ip), [])?;
                }
            }
            0b0001 => {
                if !user_exists {
                    conn.execute(&format!("INSERT INTO {} values ({}, {})", table_name, ip, "0001".to_string()), [])?;
                } else {
                    conn.execute(&format!("UPDATE {} SET stage = {} WHERE ip = {}", table_name, "0001".to_string(), ip), [])?;
                }
            }
            _ => println!("Unknown request"),
        }
    }

    Ok(())
}

fn listen_for_requests() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:3567")?;
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let network_id = "some_id".to_string(); // Example, use real network ID
                std::thread::spawn(move || {
                    handle_requests(stream, network_id).unwrap_or_else(|e| eprintln!("Error: {}", e));
                });
            }
            Err(e) => eprintln!("Connection failed: {}", e),
        }
    }
    Ok(())
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
