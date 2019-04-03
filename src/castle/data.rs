use super::Missive;
use crate::client::Kind;
use crate::db::{models, schema};
use crate::rpc::app_error;
use crate::Bus;
use crossbeam_channel::bounded;
use diesel::prelude::*;
use jsonrpc_core::{Error as RpcError, Result as RpcResult};
use log::{debug, error, info, trace};
use regex::Regex;

fn log_only(res: QueryResult<usize>) {
    if let Err(err) = res {
        error!("did not save! {:?}", err);
    }
}

#[derive(Clone, Debug)]
pub enum Topic {
    AppList {
        filter: Option<Regex>,
    },
    CreateApp {
        name: String,
        repo: String,
        build_script: Option<String>,
    },
}

pub fn request(bus: &Bus<Missive>, topic: Topic) -> RpcResult<Missive> {
    let (tx, rx) = bounded(1);
    trace!("making {:?} request to data service", topic);
    bus.send_top(Missive::DataRequest { topic, tx });
    rx.recv()
        .map_err(|_| app_error(68, "data service channel disconnect", None))?
}

pub fn data_service(bus: Bus<Missive>) {
    std::thread::Builder::new()
        .name("data service".into())
        .spawn(move || {
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
                            Topic::CreateApp {
                                name,
                                repo,
                                build_script,
                            } => create_app(&db, name, repo, build_script),
                        };

                        if let Err(err) = tx.send(data) {
                            error!("failed to send data back to {}: {:?}", source, err);
                        }
                    }
                    _ => continue,
                }
            }

            debug!("data service thread end");
        })
        .expect("failed to start data service");
}

fn db_error(err: diesel::result::Error) -> RpcError {
    use serde_json::json;
    match err {
        _ => app_error(
            800,
            "uncaught database error",
            Some(json!(format!("{}", err))),
        ),
    }
}

fn app_list(db: &PgConnection, filter: Option<Regex>) -> RpcResult<Missive> {
    use schema::apps::dsl::*;

    let mut results = apps.load::<models::App>(db).map_err(db_error)?;

    if let Some(re) = filter {
        results = results
            .into_iter()
            .filter(|app| re.is_match(&app.name))
            .collect();
    }

    Ok(Missive::AppList(results))
}

fn create_app(
    db: &PgConnection,
    name: String,
    repo: String,
    build_script: Option<String>,
) -> RpcResult<Missive> {
    let new_app = models::NewApp {
        name,
        repo,
        build_script,
    };

    Ok(Missive::App({
        use schema::apps::dsl::*;
        diesel::insert_into(apps)
            .values(&new_app)
            .get_result(db)
            .map_err(db_error)?
    }))
}
