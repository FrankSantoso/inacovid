use chrono::{Duration, Utc};
use qstring::QString;

// ------- Endpoint helpers -------- //
pub enum Endpoint {
    Perprov,
    Perkembangan,
    // Surveillance,
}

pub fn ep_val(e: Endpoint) -> String {
    match e {
        Endpoint::Perprov => "https://services5.arcgis.com/VS6HdKS0VfIhv8Ct/arcgis/rest/services/COVID19_Indonesia_per_Provinsi/FeatureServer/0/query".to_string(),
        Endpoint::Perkembangan => "https://services5.arcgis.com/VS6HdKS0VfIhv8Ct/arcgis/rest/services/Statistik_Perkembangan_COVID19_Indonesia/FeatureServer/0/query".to_string(),
        // Endpoint::Surveillance=> "covid-monitoring2.kemkes.go.id".to_string(),
    }
}

// ------ Query Param Helpers -------- //
#[derive(Debug)]
pub struct QueryParams {
    f: String,
    out_fields: String,
    pub where_query: WhereQueries,
    spatial_rel: String,
}

impl QueryParams {
    pub fn new() -> Self {
        QueryParams {
            f: "json".to_string(),
            where_query: WhereQueries::All,
            out_fields: "*".to_string(),
            spatial_rel: "esriSpatialRelIntersects".to_string(),
        }
    }
    // build query params
    pub fn query_params(&self) -> String {
        let wherev = where_val(self.where_query);
        let fields: Vec<(&str, &str)> = vec![
            ("f", self.f.as_str()),
            ("where", wherev.as_str()),
            ("returnGeometry", "false"),
            ("spatialRel", self.spatial_rel.as_str()),
            ("outFields", self.out_fields.as_str()),
            ("cacheHint", "true"),
        ];
        format!("{}", QString::new(fields))
    }

    pub fn add_queries(&self, kv: Vec<(&str, &str)>) -> String {
        let current_query = QString::from(self.query_params().as_str());
        let mut queryparams = current_query.to_pairs();
        kv.iter()
            .map(|(k, v)| queryparams.push((*k, *v)))
            .for_each(drop);
        format!("{}", QString::new(queryparams))
    }
}

#[derive(Copy, Clone, Debug)]
pub enum WhereQueries {
    Confirmed,
    Deaths,
    Recovered,
    All,
    Indonesia,
    CurrentDate(i64),
    BeforeToday(i64),
}

pub fn where_val(w: WhereQueries) -> String {
    match w {
        WhereQueries::Confirmed => "(Confirmed > 0)".to_string(),
        WhereQueries::Deaths => "(Confirmed > 0) AND (Deaths > 0)".to_string(),
        WhereQueries::Recovered => "(Confirmed > 0) AND (Recovered <> 0)".to_string(),
        WhereQueries::All => "1=1".to_string(),
        WhereQueries::Indonesia => {
            "(Provinsi = 'Indonesia') OR (Provinsi <> 'Indonesia')".to_string()
        }
        WhereQueries::CurrentDate(prefix) => {
            let new_prefix = count_prefix(prefix);
            let yesterday = add_cur_date(Duration::days(new_prefix - 1));
            let tomorrow = add_cur_date(Duration::days(new_prefix + 1));
            let now = add_cur_date(Duration::days(new_prefix));
            let wq = format!("(Tanggal>=timestamp '{} 17:00:00' AND Tanggal<=timestamp '{} 16:59:59' OR Tanggal>=timestamp '{} 16:59:59')",
                             now, tomorrow, yesterday );
            wq
        }
        WhereQueries::BeforeToday(prefix) => {
            let new_prefix = count_prefix(prefix);
            format!(
                "Tanggal<timestamp '{} 17:00:00'",
                add_cur_date(Duration::days(new_prefix))
            )
        }
    }
}

pub fn create_statistics_query(field: &str) -> String {
    format!("[{{\"statisticType\":\"sum\",\"onStatisticField\":\"{}\",\"outStatisticFieldName\":\"value\"}}]", field)
}

fn count_prefix(i: i64) -> i64 {
    let mut prefix = i;
    let cur_time = Utc::now().naive_utc().time();
    let upd_time = chrono::NaiveTime::from_hms(10, 0, 0);

    if cur_time < upd_time {
        prefix = i - 1;
    }
    prefix
}

pub fn add_cur_date(dur: Duration) -> String {
    let now = Utc::now();
    let added_now = now.checked_add_signed(dur);
    match added_now {
        None => format_date(now.naive_utc()),
        Some(d) => format_date(d.naive_utc()),
    }
}

fn format_date(d: chrono::NaiveDateTime) -> String {
    d.format("%Y-%m-%d").to_string()
}

fn format_date_with_time(d: chrono::NaiveDateTime) -> String {
    d.format("%Y-%m-%d %H:%M:%S").to_string()
}

pub fn timestamp_to_date(ts: Option<i64>) -> String {
    match ts {
        Some(secs) => {
            let date_time = chrono::NaiveDateTime::from_timestamp(secs / 1000, 0);
            format_date_with_time(date_time)
        }
        _ => format_date_with_time(Utc::now().naive_utc()),
    }
}
