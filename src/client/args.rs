use clap::{App, Arg};

pub fn arguments<'a, 'b>(name: &str) -> App<'a, 'b> {
    App::new(name)
        .version(env!("CARGO_PKG_VERSION"))
        .about("Configure, monitor, and deploy apps with Trebuchet")
        .arg(
            Arg::with_name("host")
                .long("host")
                .value_name("HOSTNAME")
                .help("Sets the Trebuchet server to connect to")
                .takes_value(true)
                .default_value("localhost"),
        )
        .arg(
            Arg::with_name("port")
                .long("port")
                .value_name("PORT")
                .help("Sets the Trebuchet server port")
                .takes_value(true)
                .default_value("9077"),
        )
        .arg(
            Arg::with_name("name")
                .long("name")
                .value_name("NAME")
                .help("Identifies to the server with a name")
                .takes_value(true)
                .default_value(&crate::HOSTNAME),
        )
        .arg(
            Arg::with_name("tags")
                .long("tags")
                .value_name("TAGS")
                .help("Identifies to the server with tags (comma-separated)")
                .value_delimiter(","),
        )
        .arg(
            Arg::with_name("v")
                .short("v")
                .multiple(true)
                .help("Sets the level of verbosity (0-5)"),
        )
        .arg(
            Arg::with_name("q")
                .short("q")
                .multiple(true)
                .help("Sets the level of quietness (0-3)"),
        )
}
