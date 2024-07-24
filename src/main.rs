use std::{env, io::{self, Write}, net::TcpStream, path::PathBuf};
use ssh2::Session;


struct CurrentPath {
    current_path: PathBuf,
}

impl CurrentPath {

    fn new(initial_path: String) -> Self {
        CurrentPath {
            current_path: PathBuf::from(initial_path),
        }
    }

    fn change_path(&mut self, new_path: String) {
        self.current_path = PathBuf::from(self.current_path.join(new_path));
    }

    // fn get_path(&self) -> &PathBuf {
    //     &self.current_path
    // }
}

fn main() {
    
    let args: Vec<String> = env::args().collect();


    if args.len() < 4 {

        println!("Uso: {} <hostname> <username> <password>", args[0]);
        return;
    }

    let mut dir =  CurrentPath::new(String::from("."));

    let host = &args[1];
    let user = &args[2];
    let passwd = &args[3];

    let tcp_connection = TcpStream::connect(format!("{}:22", host)).unwrap();
    let mut sess = Session::new().unwrap();

    sess.set_tcp_stream(tcp_connection);
    sess.handshake().unwrap();
    sess.userauth_password(user, passwd).unwrap();

    assert!(sess.authenticated());

    let sftp = sess.sftp().unwrap();

    loop {
        print!(">> ");
        io::stdout().flush().expect("Failed to flush stdout");

        let mut input = String::new();
        io::stdin().read_line(&mut input)
            .expect("Failed to read line");
        let command = input.trim();

        if command == "ls" {

            let mut directory = sftp.opendir(&dir.current_path).unwrap();

            while let Ok(content) = directory.readdir() {
                let (entry_path, _) = content;
                    println!("{}", entry_path.display());
            }
        }

        if command.starts_with("cd ") {

            let tokens: Vec<&str> = command.split_whitespace().collect();
            
            dir.change_path(tokens[1].to_string());
        }
        

        if command == "exit" {
            break;
        }
    }
}

