use clap::{App, SubCommand};
use cloud_conveyor_core::yaml::load_app_from_yaml;

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
                .author(author),
        )
        .get_matches();

    // Run the Check Command - try and load the file. If it succeeds,
    // we are good. If not, we are not good.
    if let Some(_) = matches.subcommand_matches(check_command_name) {
        match load_app_from_yaml() {
            // TODO: Write out the error if there is one.
            Ok(_) => println!("Everything is good!"),
            Err(_) => eprintln!("Everything is NOT OK!"),
        }
    }
}
