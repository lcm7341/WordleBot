pub mod wordle {
    use colored::*;
    use std::collections::HashMap;
    mod json {
        use serde::{Deserialize, Serialize};
        use std::collections::HashMap;
        use std::fs::File;
        use std::io::prelude::*;

        #[derive(Debug, Serialize, Deserialize)]
        struct T {
            s: String,
            f: f64,
        }
        pub fn write(path: &str, data: HashMap<String, f64>) -> std::io::Result<()> {
            let mut new: Vec<_> = data.iter().collect();
            new.sort_by(|a, b| a.1.partial_cmp(b.1).unwrap());
            let serialized = serde_json::to_string_pretty(&new).unwrap();
            let mut file = File::create(path)?;
            file.write_all(&serialized.as_bytes())?;
            Ok(())
        }
        pub fn read_data(path: &str) -> HashMap<String, f64> {
            let mut file = match File::open(path) {
                Ok(file) => file,
                Err(err) => {
                    println!("my error message: {}", err);
                    std::process::exit(1);
                }
            };

            let mut contents = String::new();
            file.read_to_string(&mut contents).unwrap();

            let users: Vec<(String, f64)> = serde_json::from_str(&contents).unwrap();

            let temp_tuples = users.iter().map(|x| (x.0.clone(), x.1)).collect::<Vec<_>>();
            let map_names_by_id: HashMap<_, _> = temp_tuples.into_iter().collect();

            return map_names_by_id;
        }
        pub fn read(path: &str) -> Vec<String> {
            let mut file = match File::open(path) {
                Ok(file) => file,
                Err(err) => {
                    println!("my error message: {}", err);
                    std::process::exit(1);
                }
            };

            let mut contents = String::new();
            file.read_to_string(&mut contents).unwrap();

            let users: Vec<String> = serde_json::from_str(&contents).unwrap();
            return users;
        }
    }

    fn get_words() -> Vec<String> {
        json::read("words.json")
    }

    use rayon::prelude::*;

    pub fn score_letters(list: Vec<String>) -> HashMap<String, i32> {
        let mut letters_map: HashMap<String, i32> = HashMap::new();

        for word in list.iter() {
            for (ind, letter) in word.char_indices() {
                let entry = letters_map.entry(letter.to_string()).or_insert(0);
                *entry += 1 * (5 - ind as i32);
            }
        }

        return letters_map;
    }

    pub fn score_word(word: &str, letters: HashMap<String, i32>) -> i32 {
        let mut score: i32 = 0;

        for (index, letter) in word.char_indices() {
            if word.matches(letter).count() == 1 {
                score += letters.get(&letter.to_string()).unwrap_or(&0)
                    + (word.len() as i32 - index as i32);
            }
        }

        return score;
    }

    pub fn score_all_words() -> HashMap<String, i32> {
        let mut map: HashMap<String, i32> = HashMap::new();
        let words = get_words();
        let letters = score_letters(get_words());
				
				let mut data = json::read_data("data.json");

        for word in words.iter() {
						let divide_by = *data.get(&word.to_string()).unwrap_or(&1.0);
						data.remove_entry(&word.to_string());
						let score: f64 = score_word(word, letters.clone()) as f64 / (divide_by * 100.0);
						if divide_by != 1.0 {
							//println!("{}: {}", word, divide_by);
						}
					
            map.insert(word.to_string(), (score) as i32);
        }

        map
    }

    pub fn get_accuracy(correct: &str, guess: &str) -> String {
        let mut accuracy = String::with_capacity(5);
        accuracy.push_str("-----");
        let mut correct_new = correct.to_string();

        let mut greens = Vec::with_capacity(correct.len());
        let mut yellows = Vec::with_capacity(correct.len());

        for i in 0..correct.len() {
            if &correct[i..=i] == &guess[i..=i] {
                accuracy.replace_range(i..=i, "g");
                greens.push(i);
                correct_new.replace_range(i..=i, "-");
            } else {
                yellows.push(i);
            }
        }

        //println!("accuracy: {accuracy}, correct_new: {correct_new}, greens: {:?}, yellow: {:?}", greens, yellows);

        for i in yellows.clone().into_iter() {
            let ch = &guess[i..=i];
            if let Some(pos) = correct_new.find(ch) {
                accuracy.replace_range(i..=i, "y");
				correct_new.replace_range(pos..=pos, "-");
                let c_pos = greens.iter().position(|&j| j == pos);
                if let Some(j) = c_pos {
                    greens.swap_remove(j);
                }
            } else {
                accuracy.replace_range(i..=i, "m");
            }
        }
       //println!("accuracy: {accuracy}, correct_new: {correct_new}, greens: {:?}, yellow: {:?}", greens, yellows);

        accuracy
    }

    pub fn filter(
        guess: &str,
        accuracy: &str,
        mut map: HashMap<String, i32>,
    ) -> HashMap<String, i32> {
        let word_acc = |w: (&String, &i32)| get_accuracy(w.0, guess);

        let mut to_remove = Vec::new();
        for (word, i) in map.iter() {
            if word_acc((word, i)).ne(accuracy) {
                to_remove.push(word.clone());
            }
        }
        for word in to_remove {
			//*map.get_mut(&word).unwrap() /= 2;
            map.remove(&word);
        }
		map.remove(guess);

        map
    }

    pub fn simulate_game() -> f64 {
        let mut avg_guesses: f64 = 1.00;

        let map = score_all_words();
        let mut guesses: f64 = 0.0;
        let mut times: f64 = 0.0;

        for word in map.iter() {
            let mut filtered = map.clone();
            let mut best = filtered.iter().max_by_key(|entry| entry.1).unwrap().0;

            let g: String = best.to_string();

            let mut acc = get_accuracy(&g, word.0);

            while filtered.len().ne(&1) && acc.ne(&String::from("ggggg")) {
                println!("{}", filtered.len());
                filtered = filter(best.trim(), acc.trim(), filtered.clone());

                filtered.remove(&g);
                guesses += 1.0;

                best = filtered.iter().max_by_key(|entry| entry.1).unwrap().0;
                acc = get_accuracy(&best, word.0);
            }
            times += 1.0;
            avg_guesses = guesses / times;
            println!("avg: {}", &avg_guesses);
        }

        avg_guesses
    }

    fn color_from_char(ch: u8) -> ColoredString {
        match ch as char {
            'm' => ColoredString::from("m").bright_black(),
            'y' => ColoredString::from("y").yellow(),
            'g' => ColoredString::from("g").green(),
            _ => ColoredString::from(ch.to_string().as_str()),
        }
    }

    pub fn play_single(guess: String, correct: String) {
        let mut acc = get_accuracy(&correct, &guess);
        let mut guesses = 0;
        let map = score_all_words();
        let mut filtered = map.clone();
        while acc.ne(&String::from("ggggg")) {
            let cln = filtered.clone();
            let best = match cln.iter().max_by_key(|entry| entry.1) {
                None => &correct,
                Some(n) => n.0,
            };
            filtered.remove(&best.to_string());

            filtered = filter(best.trim(), acc.trim(), filtered.clone());

            guesses += 1;
            acc = get_accuracy(&correct, &best);
            // I think ColoredString only supports one color
            let colored = format!(
                "{}{}{}{}{}",
                color_from_char(acc.as_bytes()[0]),
                color_from_char(acc.as_bytes()[1]),
                color_from_char(acc.as_bytes()[2]),
                color_from_char(acc.as_bytes()[3]),
                color_from_char(acc.as_bytes()[4])
            );
            println!("#{}. {} -> {}", &guesses, &best, colored);
        }
    }

    pub fn sim_single(guess: &str, correct: &str) -> i32 {
        let mut acc = get_accuracy(correct, guess);
        let mut guesses = 1;
        let map = score_all_words();
        let mut filtered = map.clone();
        let mut unbiased_filtered = map.clone();
        filtered = filter(guess.trim(), acc.trim(), filtered.clone());
		unbiased_filtered = filter(guess.trim(), acc.trim(), unbiased_filtered.clone());
        filtered.remove(guess);
		unbiased_filtered.remove(guess);

        if filtered.len() >= 50 && filtered.len() != 1 {
            // You really only have to compare them all to the first word
                let compare_to = filtered.iter().nth(1).unwrap().0;
                let mut shared_letters: Vec<char> = Vec::new();
                let mut unshared_letters: Vec<char> = Vec::new();
            
                for (word, _score) in filtered.iter() {
                    for (index, letter) in word.chars().enumerate() {
                        if letter == compare_to.chars().nth(index).unwrap() && !shared_letters.contains(&letter) && word != compare_to {
                            shared_letters.push(letter);
                        }
                        if letter != compare_to.chars().nth(index).unwrap() && !unshared_letters.contains(&letter) && word != compare_to {
                            unshared_letters.push(letter);
                        }
                    }
                }
            
                filtered = score_all_words();
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

        while acc != "ggggg" {
            let if_wrong = String::from("-----");
            let cloned = filtered.clone();
            let best = cloned
                .iter()
                .max_by_key(|entry| entry.1)
                .map(|(w, _)| w)
                .unwrap_or(&if_wrong);

            acc = get_accuracy(correct, best);

            filtered.remove(best);

            filtered = filter(best.trim(), acc.trim(), cloned.clone());

            guesses += 1;
        }
        guesses
    }

    use std::sync::Mutex;
    pub fn sim_all() -> HashMap<String, f64> {
        let guesses = score_all_words();
        let corrects = score_all_words();
        let mut mapped_avg: HashMap<String, f64> = HashMap::from(json::read_data("data.json"));

        println!("Initial map size: {}", &mapped_avg.len());

        let correct_vec: Vec<String> = corrects.clone().into_iter().map(|w| w.0).collect();

        for guess in guesses.into_iter() {

            let total: Mutex<f64> = Mutex::new(0.000);
            let guess_count = Mutex::new(0.0);
            let avg = Mutex::new(0.000);

            correct_vec.par_iter().for_each(|element| {
                if let Some(correct) = corrects.get_key_value(element) {
                    let sim = sim_single(&guess.0, &correct.0) as f64;
                    {
                        let mut total = total.lock().unwrap();
                        let mut guess_count = guess_count.lock().unwrap();
                        let mut avg = avg.lock().unwrap();
                        *guess_count += sim;
                        *total += 1.0;
                        *avg = ((*guess_count / *total) * 10000.0).round() / 10000.0;
                    }
                    // println!(
                    //     "{} {} {} {:.2} #{}",
                    //     &guess.0,
                    //     &correct.0,
                    //     &sim,
                    //     avg.lock().unwrap(),
                    //     total.lock().unwrap(),
                    // );
                    if *total.lock().unwrap() >= 50.000 {
                        return;
                    }
                }
            });

            println!("{:?}: {:.4}", guess, *avg.lock().unwrap());
            mapped_avg.entry(guess.0).or_insert(*avg.lock().unwrap());
            json::write(&"data.json", mapped_avg.clone()).unwrap();
            println!("{:?}", &mapped_avg.len());
        }

        mapped_avg
    }

    pub fn color_code(word: &str, confidence: i32) -> ColoredString {
        match confidence {
            0..=20 => word.red(),
            21..=50 => word.yellow(), //word.truecolor(196, 98, 16),
            51..=70 => word.bright_yellow(),
            71..=90 => word.green(),
            91..=99 => word.bright_green(),
            100 => word.bright_blue(),
            _ => word.normal(),
        }
    }

    #[cfg(test)]
    mod tests {
        #[test]
        fn test_colors() {
            let arose_worse = super::get_accuracy("worse", "arose");

            let poser_swore = super::get_accuracy("swore", "poser");

            let sassy_salsa = super::get_accuracy("salsa", "sassy");
			
            let elbow_xylyl = super::get_accuracy("elbow", "xylyl");

            let touch_clump = super::get_accuracy("touch", "clump");

            let hence_cloth = super::get_accuracy("hence", "cloth");

            assert_eq!(arose_worse, String::from("myygg"));
            assert_eq!(poser_swore, String::from("myyyy"));
            assert_eq!(sassy_salsa, String::from("ggmgm"));
			assert_eq!(elbow_xylyl, String::from("mmymm"));
			assert_eq!(touch_clump, String::from("ymgmm"));
			assert_eq!(hence_cloth, String::from("ymmmy"));
        }
    }
}
