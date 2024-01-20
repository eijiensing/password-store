use console::{style, Term};
use dialoguer::{theme::ColorfulTheme, Input, Password, Select};
use serde::{Deserialize, Serialize};
use simple_crypt::{decrypt, encrypt};
use std::{fs, io::Write, path::PathBuf};

#[derive(Serialize, Deserialize)]
struct EncryptedData<'a> {
    name: &'a str,
    encrypted: Vec<u8>,
}

fn main() {
    let term = Term::stdout();
    let key = Password::with_theme(&ColorfulTheme::default())
        .with_prompt("Enter password")
        .interact()
        .unwrap();
    term.clear_screen().unwrap();
    evaluate_password_strength(key.as_str());

    menu(key.as_bytes());
}

fn get_save_path() -> PathBuf {
    let mut path = dirs_next::config_dir().unwrap();
    path.push("password-store/encrypted"); //
    path.set_extension("json");
    path
}
fn evaluate_password_strength(password: &str) {
    let mut term = Term::stdout();
    let mut password_improvement_tips: Vec<&str> = Vec::new();
    if password.len() < 14 {
        password_improvement_tips.push("Make it at least 14 characters long.");
    }

    let has_upper = password
        .chars()
        .find(|character| return character.is_uppercase())
        .is_some();
    let has_lower = password
        .chars()
        .find(|character| return character.is_lowercase())
        .is_some();

    if !(has_upper && has_lower) {
        password_improvement_tips.push("Use a combination of lower and uppercase characters.");
    }

    if password
        .chars()
        .find(|character| return character.is_digit(10))
        .is_none()
    {
        password_improvement_tips.push("Use digits.");
    }

    if password
        .chars()
        .find(|character| return !character.is_alphanumeric())
        .is_none()
    {
        password_improvement_tips.push("Use symbols.");
    }

    if password_improvement_tips.len() > 0 {
        //potentially weak password
        println!(
            "{}",
            style("You have entered a potentially weak password. Consider these tips:")
                .yellow()
                .bold()
        );
        for tip in password_improvement_tips {
            println!(" • {}", style(tip).italic());
        }
        term.write(b"\npress any key...").unwrap();
        term.read_key().unwrap();
        term.clear_screen().unwrap();
    }
}

fn menu(key: &[u8]) {
    let term = Term::stdout();
    term.clear_screen().unwrap();
    let items = vec!["View", "Add", "Delete", "Exit"];

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("What do you want to do with your data?")
        .items(&items)
        .default(0)
        .interact()
        .unwrap();

    match selection {
        0 => view(key),
        1 => add(key),
        2 => delete(key),
        3 => return,
        _ => return,
    }
}

fn delete(key: &[u8]) {
    let mut term = Term::stdout();

    let contents = match fs::read_to_string(get_save_path()) {
        Ok(value) => value,
        Err(_) => {
            term.write(b"You have no stored passwords!").unwrap();
            term.read_key().unwrap();
            term.clear_screen().unwrap();
            menu(key);
            return;
        }
    };

    let mut all_encrypted: Vec<EncryptedData> =
        serde_json::from_str(&contents).expect("Should have been able to deserialize the file");

    if all_encrypted.len() == 0 {
        term.write(b"You have no stored passwords!").unwrap();
        term.read_key().unwrap();
        term.clear_screen().unwrap();
        menu(key);
        return;
    }

    let items = all_encrypted.iter().map(|x| x.name).collect::<Vec<&str>>();

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select data to delete")
        .items(&items)
        .default(0)
        .interact()
        .unwrap();

    all_encrypted.remove(selection);

    let serialized = serde_json::to_string(&all_encrypted).unwrap();
    fs::write(get_save_path(), serialized).expect("Should be able to create/write file");
    menu(key);
}

fn view(key: &[u8]) {
    let mut term = Term::stdout();
    let contents = match fs::read_to_string(get_save_path()) {
        Ok(value) => value,
        Err(_) => {
            term.write(b"You have no stored passwords!").unwrap();
            term.read_key().unwrap();
            term.clear_screen().unwrap();
            menu(key);
            return;
        }
    };

    let all_encrypted: Vec<EncryptedData> =
        serde_json::from_str(&contents).expect("Should have been able to deserialize the file");

    if all_encrypted.len() == 0 {
        term.write(b"You have no stored passwords!").unwrap();
        term.read_key().unwrap();
        term.clear_screen().unwrap();
        menu(key);
        return;
    }

    let items = all_encrypted.iter().map(|x| x.name).collect::<Vec<&str>>();

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select data to view")
        .items(&items)
        .default(0)
        .interact()
        .unwrap();

    let encrypted_vec = all_encrypted.get(selection).unwrap().encrypted.clone();

    match decrypt(&encrypted_vec, key) {
        Ok(decrypted) => {
            println!("{}", String::from_utf8(decrypted).unwrap());
            term.read_key().unwrap();
            menu(key);
            return;
        }
        Err(_) => {
            term.write(b"Incorrect password!").unwrap();
            term.read_key().unwrap();
            term.clear_screen().unwrap();
            menu(key);
            return;
        }
    }
}

fn add(key: &[u8]) {
    let term = Term::stdout();
    let name: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Name")
        .interact_text()
        .unwrap();

    let data: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Data")
        .interact_text()
        .unwrap();

    let encypted = encrypt(data.as_bytes(), key).expect("Data should be able to be encrypted");

    //read in the file
    let contents = match fs::read_to_string(get_save_path()) {
        Ok(value) => value,
        Err(_) => String::from("[]"),
    };

    let mut all_encrypted: Vec<EncryptedData> =
        serde_json::from_str(&contents).expect("Should have been able to deserialize the file");

    match all_encrypted.iter().find(|x| x.name == name.as_str()) {
        Some(_) => {
            println!("An entry with the name '{name}' already exists");
            term.read_key().unwrap();
            term.clear_screen().unwrap();
            add(key);
            return;
        }
        None => (),
    }

    //add created data
    let encrypted_data = EncryptedData {
        name: name.as_str(),
        encrypted: encypted,
    };

    all_encrypted.push(encrypted_data);

    //save file
    let serialized = serde_json::to_string(&all_encrypted).unwrap();
    match fs::write(get_save_path(), serialized.clone()) {
        Ok(_) => (),
        Err(_) => {
            let mut path = dirs_next::config_dir().unwrap();
            path.push("password-store");
            fs::create_dir(path)
                .expect("Should be able to create directory in configuration directory!");
            fs::write(get_save_path(), serialized).expect("Should be able to write/create file!");
        }
    };
    menu(key);
}
