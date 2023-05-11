use kupier_lang::parser::{KlangParser, Rule};
use pest::Parser;
use std::sync::atomic::AtomicUsize;
use std::time::Instant;

// https://stackoverflow.com/questions/61699050/how-can-i-make-my-rust-code-run-faster-in-parallel
// https://stackoverflow.com/questions/68547268/cannot-borrow-data-in-an-arc-as-mutable

static GLOBAL_THREAD_COUNT: AtomicUsize = AtomicUsize::new(0);

#[tokio::main]
async fn main() {
    let mut command: String = String::from("");

    loop {
        let mut prompt = "kupier >> ";
        if command.len() > 0 {
            prompt = "";
        }

        let input = user_input::get_input(prompt);

        if input == "exit" {
            break;
        }

        if input != "" {
            if command.len() > 0 {
                command.push_str("\n");
            }

            command.push_str(&input);
            continue;
        }

        let local_command = command.clone();
        command.clear();

        println!("------------------------------------------------------");
        println!("COMMAND:");
        println!("------------------------------------------------------");
        println!("{}", local_command);
        println!("------------------------------------------------------");
        println!();

        let now = Instant::now();
        let result = kupier_lang::parser::parse_query(&local_command);
        let microseconds = now.elapsed().as_micros();

        println!("Elapsed: {}μs", microseconds);

        if result.is_ok() {
            println!("{:?}", result.unwrap());
        } else {
            println!("{:?}", result.err().unwrap())
        }
    }
}

mod user_input {
    use std::io::{self, Write};
    pub fn get_input(prompt: &str) -> String {
        print!("{}", prompt);
        io::stdout().lock().flush().unwrap();

        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_goes_into_input_above) => {}
            Err(_no_updates_is_fine) => {}
        }
        input.trim().to_string()
    }
}
