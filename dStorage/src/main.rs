use std::io;

fn main() {
    loop {
        let mut choice = String::new();
        let mut file_path = String::new();

        println!("Would you like to upload or download a file? Enter 1 for upload, 2 for download, or 0 to quit:");
        io::stdin().read_line(&mut choice).expect("Sorry, unable to read your input");
        let choice = choice.trim();  // Trim the newline characters

        println!("Your choice was {choice}");

        if choice == "1" {
            println!("Choose a file to upload - enter the path to that file:");
            io::stdin().read_line(&mut file_path).expect("Sorry, unable to read your input");
            let file_path = file_path.trim();  // Trim the newline characters
            println!("You selected the file: {file_path} to upload.");
        } else if choice == "2" {
            println!("Choose a file to download - enter the pointer ID or path:");
            io::stdin().read_line(&mut file_path).expect("Sorry, unable to read your input");
            let file_path = file_path.trim();  // Trim the newline characters
            println!("You selected the file: {file_path} to download.");
        } else if choice == "0" {
            println!("Quitting the app.");
            break;
        } else {
            println!("Invalid choice, sorry!");
        }
    }
}

