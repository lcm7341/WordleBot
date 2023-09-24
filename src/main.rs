pub mod logic;

use colored::*;
use logic::wordle;
use std::io;
use std::io::Write;

fn main() {
    let map = wordle::score_all_words();

    println!(
        "Best Starter: {}",
        map.iter()
            .max_by_key(|entry| entry.1)
            .unwrap()
            .0
            .bright_blue()
    );

    let mut filtered = map.clone();
	let mut unbiased_filtered = map.clone();

    loop {
		
        print!("Enter {}: ", "Guess".green());
        io::stdout().flush().unwrap();
        let mut g = String::new();
        io::stdin().read_line(&mut g).unwrap();

        print!(
            "Enter Colors ({}, {}, {}): ",
            "m".bright_black(),
            "y".bright_yellow(),
            "g".bright_green()
        );
        io::stdout().flush().unwrap();
        let mut acc = String::new();
        io::stdin().read_line(&mut acc).unwrap();
        io::stdout().flush().unwrap();

        filtered = wordle::filter(g.trim(), acc.trim(), filtered.clone());
		unbiased_filtered = wordle::filter(g.trim(), acc.trim(), unbiased_filtered.clone());

        filtered.remove(&g);
		unbiased_filtered.remove(&g);
        
		let mut compare_to = unbiased_filtered.iter().nth(0).unwrap().0.clone();
		let mut shared_letters: Vec<char> = Vec::new();
		let mut unshared_letters: Vec<char> = Vec::new();

        for (word, _score) in unbiased_filtered.iter() {
            for (index, letter) in word.chars().enumerate() {
                if letter == compare_to.chars().nth(index).unwrap() && !shared_letters.contains(&letter) && *word != compare_to {
                    for comp_letter in compare_to.chars() {
                        if comp_letter != '-' {
                            shared_letters.push(comp_letter);
                        }
                    }
                }
                else if letter != compare_to.chars().nth(index).unwrap() && !unshared_letters.contains(&letter) && *word != compare_to {
                    unshared_letters.push(letter);
                    compare_to.replace_range(index..=index, "-");
                }
            }
        }
        
		if unbiased_filtered.len() <= 25 && unbiased_filtered.len() > 2 && unshared_letters.len() > 1 {
		// You really only have to compare them all to the first word
		
			filtered = wordle::score_all_words();
			for (word, score) in filtered.iter_mut() {
				for letter in &shared_letters {
					if word.contains(*letter) {
						*score /= 10;
					}
				}
				for letter in &unshared_letters {
					if word.contains(*letter) {
						*score *= 2;
					}
				}
			}
			
		} else {
			filtered = unbiased_filtered.clone();
		}

        if acc.trim().to_string() == String::from("ggggg") {
            filtered = map.clone();
            unbiased_filtered = map.clone();
            println!("\n{}", "New Game Started".bright_green());
            println!("\n-----------------------\n");
            continue;
        }

        match filtered.iter().max_by_key(|entry| entry.1) {
            None => println!("Empty List!"),
            Some(n) => {
                let score: f64 = f64::from(*n.1);
                let mut confidence = ((score
                    / (filtered.iter().fold(1, |sum, x| sum + x.1)) as f64)
                    * 100.0)
                    .round() as i32;
                if confidence > 100 {
                    confidence = 100;
                }
				let out = format!(
                    "{} Best guess: {} ({}% Confident, {} left) {}",
                    wordle::color_code("|", confidence),
                    wordle::color_code(&n.0, confidence),
                    &confidence,
					unbiased_filtered.len(),
                    wordle::color_code("|", confidence)
                );
                let mut dashes = String::from(" ");
                let add_amt = out.len() - 29; // random ass value

                for _i in 0..add_amt {
                    dashes += "-";
                }
                println!("{}", wordle::color_code(&dashes, confidence));
                println!("{}", out);
                println!("{}\n", wordle::color_code(&dashes, confidence));

                if filtered.len() == 1 {
                    filtered = map.clone();
                    unbiased_filtered = map.clone();
                    println!("{}", "New Game Started".bright_green());
                    println!("\n-----------------------\n");
					println!(
				        "Best Starter: {}",
				        map.iter()
				            .max_by_key(|entry| entry.1)
				            .unwrap()
				            .0
				            .bright_blue()
				    );
                    continue;
                }
            }
        }
    }
}
