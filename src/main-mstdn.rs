extern crate mammut;
extern crate toml;

use mammut::{Mastodon, Registration};
use mammut::apps::{AppBuilder, Scopes};
use std::fs::File;
use std::fs;
use std::io;
use std::io::Write;
use std::error::Error;

// #[allow(dead_code)]
fn get_mastodon_data() -> Result<Mastodon, Box<Error>> {
    if let Ok(config) = fs::read_to_string("mastodon-data.toml") {
        Ok(Mastodon::from_data(toml::from_str(&config)?))
    } else {
        register()
    }
}

fn main() -> Result<(), Box<Error>> {
    let mastodon_inst = get_mastodon_data();
    let mastodon = mastodon_inst?;
    let you = mastodon.verify_credentials().unwrap();
    println!("{:#?}", you);

    Ok(())
}

fn register() -> Result<Mastodon, Box<Error>> {
    let app = AppBuilder {
        client_name: "mammut-examples",
        redirect_uris: "urn:ietf:wg:oauth:2.0:oob",
        scopes: Scopes::Read,
        website: Some("https://github.com/Aaronepower/mammut"),
    };

    let mut registration = Registration::new("https://layer8.space");
    registration.register(app).unwrap();
    let url = registration.authorise().unwrap();

    println!("Click this link to authorize on Mastodon: {}", url);
    println!("Paste the returned authorization code: ");

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();

    let code = input.trim();
    let mastodon = registration.create_access_token(code.to_string()).unwrap();

    // Save app data for using on the next run.
    let toml = toml::to_string(&*mastodon).unwrap();
    let mut file = File::create("mastodon-data.toml").unwrap();
    file.write_all(toml.as_bytes()).unwrap();

    Ok(mastodon)
}

pub fn read_line(message: &str) -> Result<String, Box<Error>> {
    println!("{}", message);

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    Ok(input)
}
