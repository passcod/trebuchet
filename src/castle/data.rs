use super::Missive;
use crate::client::Kind;
use crate::db::{models, schema};
use crate::rpc::app_error;
use crate::Bus;
use crossbeam_channel::bounded;
use diesel::prelude::*;
use jsonrpc_core::Error as RpcError;
use log::{debug, error, info, trace};
use regex::Regex;

fn log_only(res: QueryResult<usize>) {
    if let Err(err) = res {
        error!("did not save! {:?}", err);
    }
}

#[derive(Clone, Debug)]
pub enum Topic {
    AppList { filter: Option<Regex> },
}

pub fn request(bus: &Bus<Missive>, topic: Topic) -> Result<Option<Missive>, RpcError> {
    let (tx, rx) = bounded(1);
    trace!("making {:?} request to data service", topic);
    bus.send_top(Missive::DataRequest { topic, tx });
    rx.recv()
        .map_err(|_| app_error(68, "data service channel disconnect", None))
}

pub fn data_service(bus: Bus<Missive>) {
    std::thread::spawn(move || {
        debug!("data service thread start");
        let db = crate::db::connect();

        for (source, missive) in bus.iter_with_source() {
            match missive {
                Missive::Hello {
                    app,
                    kind,
                    name,
                    tags,
                } => {
                    info!("received hello from {}", source);
                    use schema::clients;

                    let cli = models::NewClient {
                        connection: source.clone(),
                        app,
                        target: match kind {
                            Kind::Target => true,
                            Kind::Command => false,
                        },
                        name,
                        tags,
                    };

                    log_only(
                        diesel::insert_into(clients::table)
                            .values(&cli)
                            .on_conflict(clients::columns::connection)
                            .do_update()
                            .set(&cli)
                            .execute(&db),
                    )
                }
                Missive::Exit => {
                    info!("recording client exit {}", source);
                    use schema::clients;
                    log_only(
                        diesel::update(clients::table)
                            .filter(clients::columns::connection.eq(source))
                            .set(clients::columns::connected.eq(false))
                            .execute(&db),
                    )
                }
                Missive::DataRequest { topic, tx } => {
                    info!("received {:?} request from {}", topic, source);
                    let data = match topic {
                        Topic::AppList { filter } => app_list(&db, filter),
                    };

                    if let Err(err) = tx.send(data) {
                        error!("failed to send data back to {}: {:?}", source, err);
                    }
                }
                _ => continue,
            }
        }

        debug!("data service thread end");
    });
}

fn app_list(db: &PgConnection, filter: Option<Regex>) -> Option<Missive> {
    use schema::apps::dsl::*;

    let mut results = apps.load::<models::App>(db).expect("Error loading apps"); // TODO errhanld

    if let Some(re) = filter {
        results = results
            .into_iter()
            .filter(|app| re.is_match(&app.name))
            .collect();
    }

    Some(Missive::AppList(results))
}
