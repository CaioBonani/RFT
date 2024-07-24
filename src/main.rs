use std::{env, fs::File, io::{self, BufReader, BufWriter, Read, Write}, net::TcpStream, path::{Path, PathBuf}};
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
        self.current_path = self.current_path.join(new_path);
    }

    fn get_path(&self) -> &PathBuf {
        &self.current_path
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 4 {
        println!("Uso: {} <hostname> <username> <password>", args[0]);
        return;
    }

    let mut dir = CurrentPath::new(String::from("."));

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
        io::stdin().read_line(&mut input).expect("Failed to read line");
        let command = input.trim();

        if command == "ls" {

            let mut directory = sftp.opendir(dir.get_path()).unwrap();

            while let Ok(content) = directory.readdir() {
                let (entry_path, _) = content;
                println!("{}", entry_path.display());
            }

        } else if command.starts_with("cd ") {

            let tokens: Vec<&str> = command.split_whitespace().collect();
            if tokens.len() == 2 {
                dir.change_path(tokens[1].to_string());
            } else {
                println!("Uso: cd <diretório>");
            }

        } else if command.starts_with("get ") {

            let tokens: Vec<&str> = command.split_whitespace().collect();

            if tokens.len() == 3 {

                let remote_file_path = tokens[1];
                let local_directory = tokens[2];

                if Path::new(remote_file_path).is_absolute() || remote_file_path.contains('/') {

                    println!("Não use caminhos absolutos");
                    return;
                }

                let remote_path = dir.get_path().join(remote_file_path);

                // Abra o arquivo remoto
                match sftp.open(&remote_path) {

                    Ok(mut remote_file) => {

                        let mut file_content = Vec::new();

                        if remote_file.read_to_end(&mut file_content).is_ok() {

                            // Crie o caminho local para salvar o arquivo
                            let local_file_path = Path::new(local_directory).join(remote_file_path);
                            
                            if let Some(parent) = local_file_path.parent() {

                                std::fs::create_dir_all(parent).unwrap();
                            }

                            // Salve o arquivo localmente
                            let local_file = File::create(local_file_path).unwrap();
                            let mut writer = BufWriter::new(local_file);

                            writer.write_all(&file_content).unwrap();
                            println!("Arquivo baixado com sucesso!");

                        } else {

                            println!("Erro ao ler o arquivo remoto.");
                        }
                    }
                    Err(e) => println!("Erro ao abrir o arquivo remoto: {}", e),
                }

            } else {

                println!("Uso: get <arquivo_remoto> <diretório_local>");
            }

        } else if command.starts_with("put ") {

            let tokens: Vec<&str> = command.split_whitespace().collect();

            if tokens.len() == 2 {

                let local_file_path = tokens[1];
                let local_path = PathBuf::from(local_file_path);

                if !(local_path.is_absolute()) {
    
                    println!("Use caminhos absolutos para o arquivo local");
                    return;
                }


                if !local_path.exists() {
                    println!("Arquivo local não encontrado: {}", local_file_path);
                    continue;
                }

                let file_name = local_path.file_name().unwrap().to_str().unwrap();
                let remote_path = dir.get_path().join(file_name);

                let local_file = File::open(local_path).unwrap();
                let mut reader = BufReader::new(local_file);

                let mut remote_file = sftp.create(&remote_path).unwrap();
                let mut buffer = Vec::new();
                reader.read_to_end(&mut buffer).unwrap();
                remote_file.write_all(&buffer).unwrap();

                println!("Arquivo enviado com sucesso!");

            } else {

                println!("Uso: put <arquivo_local>");
            }
            

        
        } else if command == "exit" {

            break;

        } else {

            println!("Comando desconhecido: {}", command);
        }
    }
}
