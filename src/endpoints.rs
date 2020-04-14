use crate::helpers::{
    add_cur_date, create_statistics_query, ep_val, Endpoint, QueryParams, WhereQueries,
};
use crate::models;
use crate::queries::{build_headers, make_request_url};
use crate::store::PgStore;
use chrono::Duration;
use failure::Error;
use futures::stream::{FuturesOrdered, StreamExt};
use reqwest;
use serde_json;
use std::fs::File;

fn make_base_querystring(w: WhereQueries) -> QueryParams {
    let mut new_query = QueryParams::new();
    new_query.where_query = w;
    new_query
}

pub struct Request {
    client: reqwest::Client,
    pgstore: PgStore,
    jsondir: String,
}

impl Request {
    pub fn new(store: PgStore, jsondir: String) -> Self {
        Request {
            client: reqwest::Client::new(),
            pgstore: store,
            jsondir: jsondir,
        }
    }
    async fn fetch_common(
        &self,
        w: WhereQueries,
        stat: Vec<(&str, &str)>,
        endpoint: Endpoint,
    ) -> Result<String, Error> {
        let base_query = make_base_querystring(w);
        let query_string = base_query.add_queries(stat);
        let req_url = make_request_url(ep_val(endpoint).as_str(), query_string.as_str());
        match req_url {
            Ok(u) => {
                let resp = self.fetcher(u);
                match resp.await {
                    Ok(r) => Ok(r.text().await?),
                    Err(e) => return Err(e),
                }
            }
            Err(e) => return Err(e),
        }
    }

    fn get_json_val(&self, res: Result<String, Error>) -> Result<serde_json::Value, Error> {
        match res {
            Ok(r) => {
                let v: serde_json::Value = serde_json::from_str(r.as_str())?;
                Ok(v)
            }
            _ => return Err(format_err!("Failure to fetch stats data")),
        }
    }

    async fn fetcher(&self, url: String) -> Result<reqwest::Response, Error> {
        let resp = self
            .client
            .get(url.as_str())
            .headers(build_headers())
            .send()
            .await?;
        Ok(resp)
    }

    fn get_ina_covid_vec(&self, res: models::InaCovid) -> models::DataProvinsiOptVec {
        let data_prov = res
            .get_features()
            .into_iter()
            .map(|x| models::DataProvinsiOpt::new(x))
            .collect::<Vec<models::DataProvinsiOpt>>();
        models::DataProvinsiOptVec::new(&data_prov)
    }

    fn set_json_filename(&self, name: &str) -> String {
        format!(
            "{}{}-{}.json",
            self.jsondir.as_str(),
            name,
            add_cur_date(Duration::seconds(0))
        )
    }

    pub async fn fetch_daily(&self) -> Result<String, Error> {
        let results = self
            .fetch_common(
                WhereQueries::BeforeToday(0),
                vec![
                    ("orderByFields", "Tanggal asc"),
                    ("resultRecordCount", "2000"),
                    ("resultOffset", "0"),
                ],
                Endpoint::Perkembangan,
            )
            .await?;
        let ina_covid: Result<models::InaCovid, serde_json::Error> =
            serde_json::from_str(results.as_str());
        match ina_covid {
            Ok(ic) => {
                let ina_covid_vec = self.get_ina_covid_vec(ic);
                ina_covid_vec.insert_db_daily(&self.pgstore).await?;
                match serde_json::to_string_pretty(&ina_covid_vec) {
                    Ok(d) => {
                        let json_file = self.set_json_filename("daily");
                        serde_json::to_writer(&File::create(json_file.as_str())?, &d)?;
                        Ok("Daily stats succesfully stored".to_string())
                    }
                    Err(e) => Err(format_err!("Failed to produce json {}", e)),
                }
            }
            Err(e) => return Err(format_err!("Failed to serialize InaCovid json: {}", e)),
        }
    }

    pub async fn fetch_province(&self) -> Result<String, Error> {
        let results = self
            .fetch_common(
                WhereQueries::All,
                vec![
                    ("orderByFields", "Kasus_Posi desc"),
                    ("resultRecordCount", "2000"),
                    ("resultOffset", "0"),
                ],
                Endpoint::Perprov,
            )
            .await?;
        let ina_covid: Result<models::InaCovid, serde_json::Error> =
            serde_json::from_str(results.as_str());
        match ina_covid {
            Ok(ic) => {
                let ina_covid_vec = self.get_ina_covid_vec(ic);
                ina_covid_vec.insert_db_province(&self.pgstore).await?;
                match serde_json::to_string_pretty(&ina_covid_vec) {
                    Ok(d) => {
                        let json_file = self.set_json_filename("province");
                        serde_json::to_writer(&File::create(json_file.as_str())?, &d)?;
                        Ok("Province stats succesfully stored".to_string())
                    }
                    Err(e) => Err(format_err!("Failed to produce json {}", e)),
                }
            }
            Err(e) => return Err(format_err!("Failed to serialize InaCovid json: {}", e)),
        }
    }

    // -- Disabled --
    // pub async fn get_all_cases(&self) -> Result<String, Error> {
    //     let res = self.fetcher(ep_val(Endpoint::Surveillance)).await?;
    //     Ok("Surveillance cases fetched".to_string())
    // }

    pub async fn cumulative_stats(&self, prefix: i64) -> Result<String, Error> {
        // all stats
        let stats_arr = [
            create_statistics_query("Jumlah_Pasien_Meninggal"),
            create_statistics_query("Jumlah_Pasien_Sembuh"),
            create_statistics_query("Jumlah_pasien_dalam_perawatan"),
            create_statistics_query("Jumlah_Kasus_Kumulatif"),
        ];
        // fetch each results, and puts it into owned vector
        let all_stats = stats_arr
            .iter()
            .map(|x| {
                self.fetch_common(
                    WhereQueries::CurrentDate(prefix),
                    vec![("outStatistics", x.as_str())],
                    Endpoint::Perkembangan,
                )
            })
            .collect::<FuturesOrdered<_>>()
            .collect::<Vec<Result<String, Error>>>()
            .await
            .into_iter()
            .map(|i| self.get_json_val(i))
            .collect::<Vec<Result<serde_json::Value, Error>>>();
        // use slice patterns to produce json
        match &all_stats[..] {
            [Ok(d), Ok(r), Ok(p), Ok(c)] => {
                let created = add_cur_date(Duration::seconds(0));
                let new_stats = models::CovidStatistics::new(
                    c["features"][0]["attributes"]["value"].as_i64(),
                    d["features"][0]["attributes"]["value"].as_i64(),
                    r["features"][0]["attributes"]["value"].as_i64(),
                    p["features"][0]["attributes"]["value"].as_i64(),
                    Some(created),
                );
                new_stats.insert_db(&self.pgstore).await?;
                match serde_json::to_string_pretty(&new_stats) {
                    Ok(d) => {
                        let json_file = self.set_json_filename("cumulative");
                        serde_json::to_writer(&File::create(json_file.as_str())?, &d)?;
                        Ok("Cumulative stats succesfully stored".to_string())
                    }
                    Err(e) => return Err(format_err!("Failure to produce json value {}", e)),
                }
            }
            _ => return Err(format_err!("Failure to get cumulative stats")),
        }
    }
}
