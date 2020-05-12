use clap::{Arg, App, SubCommand};
use cloud_conveyor_core::yaml::{load_app_from_yaml, write_new_config_file};

// TODO: We will want to setup the version to come from cargo.toml.

fn main() {
    let version = "1.0";
    let author = "The Cloud Conveyor Team";
    let check_command_name = "check";

    // Parse the command line args.
    let matches = App::new("Cloud Conveyor")
        .version(version)
        .author(author)
        .about("Helps Check and Onbaords Services Wiht Cloud Conveyor.")
        .subcommand(
            SubCommand::with_name(check_command_name)
                .about("Checks the conveyor.yaml configuration file for anything that's wrong.")
                .version(version)
                .author(author)
        )
        .subcommand(
            SubCommand::with_name("init")
                .about("Creates a new .conveyor.yaml file for the current directory")
                .version(version)
                .author(author)
                .arg(Arg::with_name("org")
                    .help("Sets the input file to use")
                    .required(true)
                    .index(1))
                .arg(Arg::with_name("app")
                     .help("Sets the input file to use")
                     .required(true)
                     .index(2))
                .alias("i")
        )
        .get_matches();

    // Run the Check Command - try and load the file. If it succeeds,
    // we are good. If not, we are not good.
    if matches.subcommand_matches(check_command_name).is_some() {
        match load_app_from_yaml() {
            // TODO: Write out the error if there is one.
            Ok(app) => {
                println!("{:#?}", app);
                println!("Everything is good!")
            },
            Err(_) => eprintln!("Everything is NOT OK!"),
        }
    }

    if let Some(subcommand_matches) = matches.subcommand_matches("init") {
        let app = subcommand_matches.value_of("app").unwrap().to_owned();
        let org = subcommand_matches.value_of("org").unwrap().to_owned();
        write_new_config_file(app, org).unwrap();
    }
}
