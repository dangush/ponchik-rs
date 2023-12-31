use sqlx::postgres::{PgPoolOptions, PgRow};
use sqlx::{FromRow, Row};
use sqlx::{Encode, Decode, Postgres};
use std::env;

#[derive(Debug, FromRow)]
pub struct Pairing {
	pub group_channel_id: String,
	pub date_of_intro: String,
    pub meeting_status: MeetingStatus,
    pub names: Vec<String>
}

#[derive(Debug)]
pub enum MeetingStatus {
    Open,
    Scheduled,
    Closed(FinalStatus)
}

impl sqlx::Type<Postgres> for MeetingStatus {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        sqlx::postgres::PgTypeInfo::with_name("status_enum")
    }
}

#[derive(Debug)]
pub enum FinalStatus {
    Met, // Pair met
    Fail, // Pair didn't meet
    ScheduleFail, // Scheduled but never met
    Stale // No response
}

impl<'r> Decode<'r, Postgres> for MeetingStatus {
    fn decode(value: sqlx::postgres::PgValueRef<'r>) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        let status_str: &str = Decode::<Postgres>::decode(value)?;
        
        match status_str {
            "open" => Ok(MeetingStatus::Open),
            "scheduled" => Ok(MeetingStatus::Scheduled),
            "closed_met" => Ok(MeetingStatus::Closed(FinalStatus::Met)),
            "closed_no" => Ok(MeetingStatus::Closed(FinalStatus::Fail)),
            "closed_scheduled" => Ok(MeetingStatus::Closed(FinalStatus::ScheduleFail)),
            "closed_stale" => Ok(MeetingStatus::Closed(FinalStatus::Stale)),
            _ => unimplemented!(), // stability issues with core::error::Error giving me a pain in the ass, i'm not returning errors then
        }
    }
}

impl<'q> Encode<'q, Postgres> for MeetingStatus {
    fn encode_by_ref(&self, buf: &mut sqlx::postgres::PgArgumentBuffer) -> sqlx::encode::IsNull {
        match self {
            MeetingStatus::Open => buf.extend_from_slice(b"open"),
            MeetingStatus::Scheduled => buf.extend_from_slice(b"scheduled"),
            MeetingStatus::Closed(status) => {
                match status {
                    FinalStatus::Met => buf.extend_from_slice(b"closed_met"),
                    FinalStatus::Fail => buf.extend_from_slice(b"closed_no"),
                    FinalStatus::ScheduleFail => buf.extend_from_slice(b"closed_scheduled"),
                    FinalStatus::Stale => buf.extend_from_slice(b"closed_stale"),
                }
            },
        }
        sqlx::encode::IsNull::No
    }

    fn size_hint(&self) -> usize {
        0
    }
}

pub async fn db_init() -> Result<sqlx::Pool<Postgres>, sqlx::Error> {
    dotenv::dotenv().ok();

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(env::var("DATABASE_URL").unwrap().as_str())
        .await?;

    Ok(pool)
}

pub async fn db_insert_list(pool: &sqlx::Pool<Postgres>, pairings: Vec<Pairing>) -> Result<(), sqlx::Error> {
    // let test_pair = Pairing { group_channel_id: "U2123".to_string(), date_of_intro: "19-2-23".to_string(), meeting_status: MeetingStatus::Open, names: vec!["dan".to_string(), "gurnoor".to_string()] };
    
    for pairing in pairings {
        sqlx::query("INSERT INTO ponchiks (group_channel_id, meeting_status, date_of_intro, names) VALUES ($1, $2, $3, $4)")
        .bind(pairing.group_channel_id)
        .bind(pairing.meeting_status)
        .bind(pairing.date_of_intro)
        .bind(pairing.names) // sus unwrap
        .execute(pool)
        .await?;

    }

    Ok(())
}

pub async fn db_insert_single(pool: &sqlx::Pool<Postgres>, pairing: Pairing) -> Result<(), sqlx::Error> {
    // let test_pair = Pairing { group_channel_id: "U2123".to_string(), date_of_intro: "19-2-23".to_string(), meeting_status: MeetingStatus::Open, names: vec!["dan".to_string(), "gurnoor".to_string()] };

    sqlx::query("INSERT INTO ponchiks (group_channel_id, meeting_status, date_of_intro, names) VALUES ($1, $2, $3, $4)")
        .bind(pairing.group_channel_id)
        .bind(pairing.meeting_status)
        .bind(pairing.date_of_intro)
        .bind(pairing.names) // sus unwrap
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn db_read_by_groupid(pool: &sqlx::Pool<Postgres>, channel_id: String) -> Result<Pairing, sqlx::Error> {
    let select_query = 
        sqlx::query_as::<_, Pairing>("SELECT * FROM ponchiks WHERE group_channel_id = ($1)")
        .bind(channel_id);

    let pairing: Pairing = select_query.fetch_one(pool).await?;

    Ok(pairing)
}

pub async fn db_find_all_status(pool: &sqlx::Pool<Postgres>, status: MeetingStatus) -> Result<Vec<Pairing>, sqlx::Error> {
    let select_query = 
        sqlx::query_as::<_, Pairing>("SELECT * FROM ponchiks WHERE meeting_status = ($1)")
        .bind(status);

    let pairings: Vec<Pairing> = select_query.fetch_all(pool).await?;

    Ok(pairings)
}

// TODO: consider checking old status before updating
pub async fn db_update_status(pool: &sqlx::Pool<Postgres>, channel_id: String, new_status: MeetingStatus) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE ponchiks SET meeting_status = $1 WHERE group_channel_id = $2")
        .bind(new_status)
        .bind(channel_id)
        .execute(pool)
        .await?;

    Ok(())
}