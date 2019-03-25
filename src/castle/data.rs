use super::Missive;
use crate::client::Kind;
use crate::db::models;
use crate::db::schema;
use crate::Bus;
use diesel::prelude::*;
use log::{debug, error, info};

fn log_only(res: QueryResult<usize>) {
    if let Err(err) = res {
        error!("did not save! {:?}", err);
    }
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
                _ => continue,
            }
        }

        debug!("data service thread end");
    });
}
