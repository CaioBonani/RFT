mod sftp_client;

use std::env;
use sftp_client::{commands, current_path::CurrentPath};

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

    match commands::connect_ssh(host, user, passwd) {

        Ok((sftp, _)) => {
            
            loop {
                if !commands::execute_command(&sftp, &mut dir) {
                    break;
                }
            }
        }
        Err(e) => {
            eprint!("{}", e);
        }
    };
}
