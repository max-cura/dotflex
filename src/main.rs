#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

mod dotflex;
mod driver;

extern crate clap;
use clap::{Arg, ArgGroup, App};

use dotflex::{common, util};

fn main() {
    let mut cli_commands : Vec<App> = Vec::new();
    cli_commands.push(App::new("bind")
        .about("controls automated binding of files")
        .arg(Arg::new("feature")
            .takes_value(true)
            .required(true)
            .index(1)
            .about("which feature to bind files to"))
        .arg(Arg::new("files")
            .short('f')
            .long("file")
            .multiple(true)
            .takes_value(true)
            // doesn't work properly with ArgMatches::grouped_values_of
            //.max_values(2)
            .about("files to add to local repository")));
    cli_commands.push(App::new("rebind")
        .about("rebinds specified files")
        .arg(Arg::new("feature")
            .takes_value(true)
            .required(true)
            .index(1)
            .about("which feature to bind files to"))
        .arg(Arg::new("files")
            .index(2)
            .multiple(true)
            .takes_value(true)
            .about("files to rebind")));
    // cli_commands.push(App::new("remove")
    //     .about("controls automated un-binding of files")
    //     .arg(Arg::new("files")
    //         .multiple(true)
    //         .required(true)
    //         .about("files to remove from local repository")
    //         .index(1)));
    cli_commands.push(App::new("upsync")
        .about("uploads dotfiles to repo specified with init"));
    cli_commands.push(App::new("downsync")
        .about("downloads dotfiles from repo specified with init"));
    cli_commands.push(App::new("init")
        .about("sets up remote repository for dotfiles")
        .arg(Arg::new("git")
            .long("git")
            .takes_value(true)
            .about("use git repository"))
        .group(ArgGroup::new("remote")
            .required(true)
            .args(&["git"])));
    cli_commands.push(App::new("feature")
        .about("activate and deactivate features")
        // .arg(Arg::new("only-files")
        //     .long("only-files")
        //     .about("don't run any install scripts, only move files"))
        .arg(Arg::new("enable")
            .short('e')
            .long("enable")
            .takes_value(true)
            .multiple(true)
            .number_of_values(1)
            .about("enable a feature"))
        // .arg(Arg::new("disable")
        //     .short('d')
        //     .long("disable")
        //     .takes_value(true)
        //     .multiple(true)
        //     .number_of_values(1)
        //     .about("disable a feature"))
        );

    let cli_args = App::new("dotflex")
        .version("0.1.0")
        .author("Maximilien Angelo Cura, Aditya Saligrama")
        .about("Manages your dotfiles across machines")
        .arg(Arg::new("verbose")
            .short('v')
            .about("show verbose output"))
        .subcommands(cli_commands)
        .get_matches();
    
    let use_verbose = cli_args.is_present("verbose");
    common::set_output_verbosity(use_verbose);

    match cli_args.subcommand() {
        Some(("bind", subcli_args)) => {
            driver::bind(subcli_args)
        },
        Some(("rebind", subcli_args)) => {
            driver::rebind(subcli_args)
        }
        Some(("remove", subcli_args)) => {},
        Some(("upsync", subcli_args)) => {
            driver::upsync(subcli_args)
        },
        Some(("downsync", subcli_args)) => {
            driver::downsync(subcli_args)
        },
        Some(("init", subcli_args)) => {
            driver::init(subcli_args)
        },
        Some(("feature", subcli_args)) => {
            // let enables : Vec<&'_ str> = subcli_args
            //     .values_of("enable")
            //     .unwrap_or(clap::Values::default())
            //     .collect();
            // let disables : Vec<&'_ str> = subcli_args
            //     .values_of("disable")
            //     .unwrap_or(clap::Values::default())
            //     .collect();
            driver::enable(subcli_args)
        },
        _ => {
            driver::report_status();
        }
    }
    println!("Done.");
}
