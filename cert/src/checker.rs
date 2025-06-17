use chrono::{DateTime, FixedOffset, ParseResult};
use entity::cert::{ActiveModel as CertModel, ActiveModel, Model as Cert, Model};
use hostfile::parse_file;
use once_cell::sync::Lazy;
use openssl::asn1::Asn1TimeRef;
use openssl::ssl::{SslConnector, SslMethod};
use regex::Regex;
use sea_orm::{ActiveModelTrait, ConnectOptions};
use sea_orm::{Database, IntoActiveModel};
use std::error::Error;
use std::net::TcpStream;
use std::path::Path;
use tokio::time::{Duration, interval};
use url::Url;

pub async fn run() {
    let mut interval = interval(Duration::from_secs(60));
    loop {
        interval.tick().await;
        update_cert_info().await;
        println!("Updated cert info");
    }
}

async fn update_cert_info() {
    let hosts = parse_hosts("./../hosts".to_owned());
    if let Ok(hostnames) = hosts {
        let tasks: Vec<_> = hostnames
            .into_iter()
            .map(|h| tokio::spawn(collect_certs(h)))
            .collect();

        let certs: Vec<Cert> = futures::future::join_all(tasks)
            .await
            .into_iter()
            .filter_map(|t| t.unwrap().ok())
            .collect();

        println!("{:?}", certs);

        save(certs).await;
    }
}

async fn save(certs: Vec<Cert>) {
    // todo move to struct
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

    let db = Database::connect(db_opts).await.unwrap();
    let tasks = certs
        .into_iter()
        .map(|c: Model| c.into_active_model().insert(&db))
        .collect::<Vec<_>>();

    for task in tasks {
        task.await.unwrap();
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
        name: host,
        // todo remove
        alias: "".to_string(),
        valid_from: to_datetime(cert.not_before())?.to_utc(),
        valid_to: to_datetime(cert.not_after())?.to_utc(),
        ..Default::default()
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
        assert_eq!(parse_hosts("./../hosts".to_owned()).unwrap().len(), 90);
    }

    #[test]
    fn test_check_certs() {
        update_cert_info();

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
