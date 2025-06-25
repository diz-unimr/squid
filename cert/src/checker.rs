use chrono::{DateTime, FixedOffset, ParseResult, Utc};
use entity::cert::{ActiveModel as Cert, ActiveModel, Entity};
use hostfile::parse_file;
use once_cell::sync::Lazy;
use openssl::asn1::Asn1TimeRef;
use openssl::ssl::{SslConnector, SslMethod};
use regex::Regex;
use sea_orm::{ConnectOptions, EntityTrait, NotSet, Set};
use sea_orm::{Database, sea_query};
use std::error::Error;
use std::net::TcpStream;
use std::path::Path;
use tokio::time::{Duration, interval};
use url::Url;

#[derive(Clone)]
pub struct Checker {
    interval: Duration,
    connection: ConnectOptions,
}

impl Checker {
    pub fn new() -> Checker {
        let mut db_opts = ConnectOptions::new("sqlite://db.sqlite?mode=rwc");
        db_opts
            .max_connections(100)
            .min_connections(5)
            .connect_timeout(Duration::from_secs(8))
            .acquire_timeout(Duration::from_secs(8))
            .idle_timeout(Duration::from_secs(8))
            .max_lifetime(Duration::from_secs(8))
            .sqlx_logging(true);
        // .sqlx_logging_level(log::LevelFilter::Info);
        Checker {
            interval: Duration::from_secs(30),
            connection: db_opts,
        }
    }

    pub async fn run(&self) {
        let mut interval = interval(self.interval);
        loop {
            interval.tick().await;
            self.update_cert_info().await;
            println!("Updated cert info");
        }
    }

    async fn update_cert_info(&self) {
        let hosts = parse_hosts("./hosts.example".to_owned());
        match hosts {
            Ok(names) => {
                let tasks: Vec<_> = names
                    .into_iter()
                    .map(|h| tokio::spawn(collect_certs(h)))
                    .collect();

                let certs: Vec<Cert> = futures::future::join_all(tasks)
                    .await
                    .into_iter()
                    .filter_map(|t| t.unwrap().ok())
                    .collect();

                self.save(certs).await;
            }
            Err(e) => {
                println!("{}", e)
            }
        }
    }

    async fn save(&self, certs: Vec<Cert>) {
        let db = Database::connect(self.connection.clone()).await.unwrap();
        let tasks = certs.into_iter().map(|c: ActiveModel| {
            Entity::insert(c)
                .on_conflict(
                    sea_query::OnConflict::column(entity::cert::Column::Name)
                        .update_columns([
                            entity::cert::Column::ValidFrom,
                            entity::cert::Column::ValidTo,
                        ])
                        .to_owned(),
                )
                .exec_without_returning(&db)
        });

        for task in tasks {
            task.await.unwrap();
        }
    }
}

async fn collect_certs(host: String) -> Result<Cert, Box<dyn Error + Send + Sync>> {
    let parsed = Url::parse(format!("https://{}", host).as_str())?;
    let port = parsed.port_or_known_default().ok_or("No port")?;
    let tcp = TcpStream::connect(format!("{}:{}", host, port))?;

    let connector = SslConnector::builder(SslMethod::tls())?.build();
    let stream = connector.connect(host.as_str(), tcp)?;
    let cert = stream.ssl().peer_certificate().ok_or("No certificate")?;

    Ok(Cert {
        id: NotSet,
        name: Set(host),
        valid_from: Set(to_datetime(cert.not_before())?.to_utc()),
        valid_to: Set(to_datetime(cert.not_after())?.to_utc()),
        updated: Set(Utc::now()),
    })
}

fn to_datetime(asn1: &Asn1TimeRef) -> ParseResult<DateTime<FixedOffset>> {
    static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(GMT)$").unwrap());
    // replace trailing GMT with UTC offset
    let test = RE.replace(asn1.to_string().as_str(), "+0000").to_string();

    DateTime::parse_from_str(test.as_str(), "%b %d %H:%M:%S %Y %z")
}

fn parse_hosts(file_path: String) -> Result<Vec<String>, String> {
    let path = Path::new(&file_path);
    let hosts = parse_file(path)?;

    Ok(hosts.iter().map(|host| host.names[0].to_owned()).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{NaiveDate, TimeZone};
    use openssl::asn1::Asn1Time;

    #[test]
    fn test_parse_hosts() {
        assert_eq!(
            parse_hosts("./../hosts.example".to_owned()).unwrap().len(),
            2
        );
    }

    #[tokio::test]
    async fn test_check_certs() {
        Checker::new().update_cert_info().await;

        assert_eq!(true, true);
    }

    #[test]
    fn test_parse_date() {
        let date_str = "20251219235959Z";
        let date = Asn1Time::from_str(date_str).unwrap();
        let dt = to_datetime(date.as_ref());

        assert_eq!(
            dt,
            Ok(FixedOffset::east_opt(0)
                .unwrap()
                .from_local_datetime(
                    &NaiveDate::from_ymd_opt(2025, 12, 19)
                        .unwrap()
                        .and_hms_opt(23, 59, 59)
                        .unwrap()
                )
                .unwrap())
        );
    }
}
