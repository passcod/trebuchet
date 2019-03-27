use clap::{App, Arg};

pub fn arguments<'a, 'b>() -> App<'a, 'b> {
    App::new("Trebuchet castle server")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Run the Trebuchet lightweight deploy orchestrator")
        .arg(
            Arg::with_name("host")
                .long("host")
                .value_name("HOSTNAME")
                .help("Sets the host/IP to listen on")
                .takes_value(true)
                .default_value("0.0.0.0"),
        )
        .arg(
            Arg::with_name("port")
                .long("port")
                .value_name("PORT")
                .help("Sets the port to listen on")
                .takes_value(true)
                .default_value("9077"),
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
