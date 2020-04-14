use crate::helpers::timestamp_to_date;
use crate::store::PgStore;
use failure::Error;
use sqlx;

#[derive(Debug, Serialize, Deserialize)]
pub struct InaCovid {
    #[serde(rename = "objectIdFieldName")]
    object_id_field_name: Option<String>,
    #[serde(rename = "uniqueIdField")]
    unique_id_field: Option<UniqueIdField>,
    #[serde(rename = "globalIdFieldName")]
    global_id_field_name: Option<String>,
    #[serde(rename = "geometryType")]
    geometry_type: Option<String>,
    #[serde(rename = "spatialReference")]
    spatial_reference: Option<SpatialReference>,
    fields: Option<Vec<Field>>,
    features: Option<Vec<Feature>>,
}

impl InaCovid {
    pub fn get_features(&self) -> Vec<Feature> {
        let cloned = self.features.clone();
        match cloned {
            None => vec![],
            Some(x) => x,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Feature {
    attributes: Option<Attributes>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Attributes {
    #[serde(rename = "FID")]
    fid: Option<i64>,
    #[serde(rename = "Kode_Provi")]
    kode_provi: Option<i64>,
    #[serde(rename = "Provinsi")]
    provinsi: Option<String>,
    #[serde(rename = "Kasus_Posi")]
    kasus_posi: Option<i64>,
    #[serde(rename = "Kasus_Semb")]
    kasus_semb: Option<i64>,
    #[serde(rename = "Kasus_Meni")]
    kasus_meni: Option<i64>,
    #[serde(rename = "Value")]
    value: Option<i64>,
    #[serde(rename = "Hari_ke")]
    pub hari_ke: Option<i64>,
    #[serde(rename = "Tanggal")]
    pub tanggal: Option<i64>,
    #[serde(rename = "Jumlah_Kasus_Baru_per_Hari")]
    pub jumlah_kasus_baru_per_hari: Option<i64>,
    #[serde(rename = "Jumlah_Kasus_Kumulatif")]
    pub jumlah_kasus_kumulatif: Option<i64>,
    #[serde(rename = "Jumlah_pasien_dalam_perawatan")]
    pub jumlah_pasien_dalam_perawatan: Option<i64>,
    #[serde(rename = "Persentase_Pasien_dalam_Perawatan")]
    pub persentase_pasien_dalam_perawatan: Option<f64>,
    #[serde(rename = "Jumlah_Pasien_Sembuh")]
    pub jumlah_pasien_sembuh: Option<i64>,
    #[serde(rename = "Persentase_Pasien_Sembuh")]
    pub persentase_pasien_sembuh: Option<f64>,
    #[serde(rename = "Jumlah_Pasien_Meninggal")]
    pub jumlah_pasien_meninggal: Option<i64>,
    #[serde(rename = "Persentase_Pasien_Meninggal")]
    pub persentase_pasien_meninggal: Option<f64>,
    #[serde(rename = "Jumlah_Kasus_Sembuh_per_Hari")]
    pub jumlah_kasus_sembuh_per_hari: Option<i64>,
    #[serde(rename = "Jumlah_Kasus_Meninggal_per_Hari")]
    pub jumlah_kasus_meninggal_per_hari: Option<i64>,
    #[serde(rename = "Jumlah_Kasus_Dirawat_per_Hari")]
    pub jumlah_kasus_dirawat_per_hari: Option<i64>,
    #[serde(rename = "Kasus_Sedang_Investigasi_Lapangan")]
    pub kasus_sedang_investigasi_lapangan: Option<i64>,
    #[serde(rename = "Pembaruan_Terakhir")]
    pub pembaruan_terakhir: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataProvinsiOptVec {
    provinsi_vec: Vec<Option<IndoCovidStats>>,
}

impl DataProvinsiOptVec {
    pub fn new(vals: &Vec<DataProvinsiOpt>) -> Self {
        let mut indo_covid_vec: Vec<Option<IndoCovidStats>> = Vec::new();
        vals.clone()
            .into_iter()
            .map(|opt| indo_covid_vec.push(opt.indo_covid_stats))
            .for_each(drop);
        DataProvinsiOptVec {
            provinsi_vec: indo_covid_vec,
        }
    }
    pub async fn insert_db_daily(&self, store: &PgStore) -> Result<(), Error> {
        let mut tx = store.get_tx().await?;
        for p in self.provinsi_vec.iter() {
            match p {
                Some(prov) => {
                    sqlx::query!(
            r#"
                INSERT INTO covid_daily(day, date, new_cases_per_day, cumulative_cases, 
                    under_treatment, under_treatment_per_day, under_treatment_percentage, recovered, recovered_per_day, 
                    recovered_percentage, deaths, deaths_per_day, deaths_percentage, latest_update)
                VALUES($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
                ON CONFLICT ON CONSTRAINT covid_daily_date_key DO UPDATE SET existed = true
            "#,
            prov.day, prov.date, prov.new_cases_per_day, prov.cumulative_cases, prov.under_treatment, prov.under_treatment_per_day, prov.under_treatment_percentage, prov.recovered, prov.recovered_per_day,
            prov.recovered_percentage, prov.deaths, prov.deaths_per_day, prov.deaths_percentage, prov.latest_update,
        ).execute(&mut tx).await?;
                }
                None => continue,
            }
        }
        tx.commit().await?;
        Ok(())
    }
    pub async fn insert_db_province(&self, store: &PgStore) -> Result<(), Error> {
        let mut tx = store.get_tx().await?;
        for p in self.provinsi_vec.iter() {
            match p {
                Some(prov) => {
                    let date_only = prov.date.as_ref().unwrap();
                    let prov_date =
                        format!("{}_{}", prov.provinsi.as_ref().unwrap(), &date_only[..9],);
                    sqlx::query!(r#"
                        INSERT INTO covid_province(province_id, date, provinsi, positif, sembuh, meninggal, prov_and_date)
                        VALUES($1, $2, $3, $4, $5, $6, $7)
                        ON CONFLICT ON CONSTRAINT covid_province_prov_and_date_key DO UPDATE SET existed = true
                    "#,
                    prov.province_id, prov.date, prov.provinsi, prov.positif, prov.sembuh, prov.meninggal, prov_date)
                    .execute(&mut tx).await?;
                }
                None => continue,
            }
        }
        tx.commit().await?;
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataProvinsiOpt {
    indo_covid_stats: Option<IndoCovidStats>,
}

impl DataProvinsiOpt {
    pub fn new(feat: Feature) -> Self {
        DataProvinsiOpt {
            indo_covid_stats: Some(IndoCovidStats::new(
                feat.attributes.expect("Attributes are nil"),
            )),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IndoCovidStats {
    #[serde(rename = "ProvinceId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub province_id: Option<i64>,
    #[serde(rename = "Day")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub day: Option<i64>,
    #[serde(rename = "Date")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,
    #[serde(rename = "NewCasesPerDay")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_cases_per_day: Option<i64>,
    #[serde(rename = "CumulativeCases")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cumulative_cases: Option<i64>,
    #[serde(rename = "UnderInvestigation")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub under_investigation: Option<i64>,
    #[serde(rename = "UnderTreatment")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub under_treatment: Option<i64>,
    #[serde(rename = "UnderTreatmentPercentage")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub under_treatment_percentage: Option<f64>,
    #[serde(rename = "Recovered")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recovered: Option<i64>,
    #[serde(rename = "RecoveredPercentage")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recovered_percentage: Option<f64>,
    #[serde(rename = "Deaths")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deaths: Option<i64>,
    #[serde(rename = "DeathsPercentage")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deaths_percentage: Option<f64>,
    #[serde(rename = "RecoveredPerDay")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recovered_per_day: Option<i64>,
    #[serde(rename = "DeathsPerDay")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deaths_per_day: Option<i64>,
    #[serde(rename = "UnderTreatmentPerDay")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub under_treatment_per_day: Option<i64>,
    #[serde(rename = "Latestupdate")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latest_update: Option<String>,
    #[serde(rename = "Provinsi")]
    #[serde(skip_serializing_if = "Option::is_none")]
    provinsi: Option<String>,
    #[serde(rename = "Positif")]
    #[serde(skip_serializing_if = "Option::is_none")]
    positif: Option<i64>,
    #[serde(rename = "Sembuh")]
    #[serde(skip_serializing_if = "Option::is_none")]
    sembuh: Option<i64>,
    #[serde(rename = "Meninggal")]
    #[serde(skip_serializing_if = "Option::is_none")]
    meninggal: Option<i64>,
}

impl IndoCovidStats {
    pub fn new(attr: Attributes) -> Self {
        let date = timestamp_to_date(attr.tanggal);
        let latest_update = timestamp_to_date(attr.pembaruan_terakhir);
        IndoCovidStats {
            province_id: attr.kode_provi,
            provinsi: attr.provinsi,
            positif: attr.kasus_posi,
            sembuh: attr.kasus_semb,
            meninggal: attr.kasus_meni,
            date: Some(date),
            day: attr.hari_ke,
            new_cases_per_day: attr.jumlah_kasus_baru_per_hari,
            cumulative_cases: attr.jumlah_kasus_kumulatif,
            under_investigation: attr.kasus_sedang_investigasi_lapangan,
            under_treatment: attr.jumlah_pasien_dalam_perawatan,
            under_treatment_per_day: attr.jumlah_kasus_dirawat_per_hari,
            under_treatment_percentage: attr.persentase_pasien_dalam_perawatan,
            recovered: attr.jumlah_pasien_sembuh,
            recovered_per_day: attr.jumlah_kasus_sembuh_per_hari,
            recovered_percentage: attr.persentase_pasien_sembuh,
            deaths: attr.jumlah_pasien_meninggal,
            deaths_per_day: attr.jumlah_kasus_meninggal_per_hari,
            deaths_percentage: attr.persentase_pasien_meninggal,
            latest_update: Some(latest_update),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Field {
    name: Option<String>,
    #[serde(rename = "type")]
    field_type: Option<String>,
    alias: Option<String>,
    #[serde(rename = "sqlType")]
    sql_type: Option<String>,
    domain: Option<serde_json::Value>,
    #[serde(rename = "defaultValue")]
    default_value: Option<serde_json::Value>,
    length: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SpatialReference {
    wkid: Option<i64>,
    #[serde(rename = "latestWkid")]
    latest_wkid: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UniqueIdField {
    name: Option<String>,
    #[serde(rename = "isSystemMaintained")]
    is_system_maintained: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CovidStatistics {
    #[serde(rename = "TotalCases")]
    pub total_cases: Option<i64>,
    #[serde(rename = "Deaths")]
    pub deaths: Option<i64>,
    #[serde(rename = "Recovered")]
    pub recovered: Option<i64>,
    #[serde(rename = "Pdp")]
    pub pdp: Option<i64>,
    #[serde(rename = "Date")]
    pub created: Option<String>,
}

impl CovidStatistics {
    pub fn new(
        total_cases: Option<i64>,
        deaths: Option<i64>,
        recovered: Option<i64>,
        pdp: Option<i64>,
        date: Option<String>,
    ) -> Self {
        CovidStatistics {
            total_cases: total_cases,
            deaths: deaths,
            recovered: recovered,
            pdp: pdp,
            created: date,
        }
    }
    pub async fn insert_db(&self, store: &PgStore) -> Result<(), Error> {
        let mut tx = store.get_tx().await?;
        sqlx::query!(
            r#"
                INSERT INTO covid_stats (deaths, total_cases, recovered, pdp, at_date)
                VALUES ($1, $2, $3, $4, $5)
                ON CONFLICT ON CONSTRAINT covid_stats_at_date_key DO UPDATE SET existed = true
            "#,
            self.deaths,
            self.total_cases,
            self.recovered,
            self.pdp,
            self.created
        )
        .execute(&mut tx)
        .await?;
        tx.commit().await?;
        Ok(())
    }
}
