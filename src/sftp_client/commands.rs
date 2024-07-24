use std::error::Error;
use std::io::{self, BufReader, BufWriter, Read, Write};
use std::net::TcpStream;
use std::path::Path;
use ssh2::{Session, Sftp};
use crate::sftp_client::current_path::CurrentPath;
use std::result::Result;

pub fn connect_ssh(host: &str, user: &str, passwd: &str) -> Result<(Sftp, Session), Box<dyn Error>> {

    let tcp_connection = TcpStream::connect(format!("{}:22", host)).map_err(|e| Box::new(e) as Box<dyn Error>)?;

    let mut sess = Session::new().map_err(|e| Box::new(e) as Box<dyn Error>)?;

    sess.set_tcp_stream(tcp_connection);
    sess.handshake()?;

    sess.userauth_password(user, passwd)?;

    let sftp = sess.sftp().map_err(|e| Box::new(e) as Box<dyn Error>)?;
    
    Ok((sftp, sess))
}

pub fn execute_command(sftp: &ssh2::Sftp, dir: &mut CurrentPath) -> bool {

    print!(">> ");
    io::stdout().flush().expect("Failed to flush stdout");

    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read line");
    let command = input.trim();

    match command {

        "ls" => {
            list_files(sftp, dir);
        }

        "exit" => {
            return false;
        }
        
        _ if command.starts_with("cd ") => {
            change_directory(command, dir);
        }
        
        _ if command.starts_with("get ") => {
            get_file(command, sftp, dir);
        }

        _ if command.starts_with("put ") => {
            put_file(command, sftp, dir);
        }

        _ => {
            println!("Comando desconhecido: {}", command);
        }
    }

    true
}

fn list_files(sftp: &ssh2::Sftp, dir: &CurrentPath) {
    let mut directory = sftp.opendir(dir.get_path()).unwrap();
    while let Ok(content) = directory.readdir() {
        let (entry_path, _) = content;
        println!("{}", entry_path.display());
    }
}

fn change_directory(command: &str, dir: &mut CurrentPath) {
    let tokens: Vec<&str> = command.split_whitespace().collect();
    if tokens.len() == 2 {
        dir.change_path(tokens[1].to_string());
    } else {
        println!("Uso: cd <diretório>");
    }
}

fn get_file(command: &str, sftp: &ssh2::Sftp, dir: &CurrentPath) {
    let tokens: Vec<&str> = command.split_whitespace().collect();
    if tokens.len() == 3 {
        let remote_file_path = tokens[1];
        let local_directory = tokens[2];
        if Path::new(remote_file_path).is_absolute() || remote_file_path.contains('/') {
            println!("Não use caminhos absolutos");
            return;
        }
        let remote_path = dir.get_path().join(remote_file_path);
        match sftp.open(&remote_path) {
            Ok(mut remote_file) => {
                let mut file_content = Vec::new();
                if remote_file.read_to_end(&mut file_content).is_ok() {
                    let local_file_path = Path::new(local_directory).join(remote_file_path);
                    if let Some(parent) = local_file_path.parent() {
                        std::fs::create_dir_all(parent).unwrap();
                    }
                    let local_file = std::fs::File::create(local_file_path).unwrap();
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
}

fn put_file(command: &str, sftp: &ssh2::Sftp, dir: &CurrentPath) {
    let tokens: Vec<&str> = command.split_whitespace().collect();
    if tokens.len() == 2 {
        let local_file_path = tokens[1];
        let local_path = std::path::PathBuf::from(local_file_path);
        if !local_path.is_absolute() {
            println!("Use caminhos absolutos para o arquivo local");
            return;
        }
        if !local_path.exists() {
            println!("Arquivo local não encontrado: {}", local_file_path);
            return;
        }
        let file_name = local_path.file_name().unwrap().to_str().unwrap();
        let remote_path = dir.get_path().join(file_name);
        let local_file = std::fs::File::open(local_path).unwrap();
        let mut reader = BufReader::new(local_file);
        let mut remote_file = sftp.create(&remote_path).unwrap();
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer).unwrap();
        remote_file.write_all(&buffer).unwrap();
        println!("Arquivo enviado com sucesso!");
    } else {
        println!("Uso: put <arquivo_local>");
    }
}
