pub(crate) mod cli {
    use std::io::{self, Write};

    #[derive(Clone)]
    pub struct CLIOption {
        pub(crate) title: String,
        pub(crate) artist: Option<String>,
        pub(crate) id: String,
    }

    pub fn get_user_input() -> String {
        io::stdout().flush().expect("Failed to flush stdout");

        // Read the input
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");

        // Trim whitespace and return
        let query_base = input.trim().to_string();

        query_base
    }

    pub fn present_options(options: Vec<CLIOption>) -> Option<CLIOption> {
        for (index, option) in options.iter().enumerate() {
            let mut artist_str = String::new();

            if let Some(artist) = &option.artist {
                artist_str = format!(" by {}", artist);
            }

            println!(
                "{}. {}{}", // Damn Rust for not letting me pass a string template.
                index + 1,
                option.title,
                artist_str,
            );
        }

        let mut input = String::new();

        io::stdin().read_line(&mut input).unwrap();

        if let Ok(index) = input.trim().parse::<usize>() {
            if index > 0 && index <= options.len() {
                return Some(options[index - 1].clone());
            }
        }

        None
    }
}
