use sqlx::postgres::{PgPoolOptions, PgRow};
use sqlx::{FromRow, Row};
use sqlx::{Encode, Decode, Postgres};
use std::env;

use tracing_subscriber;
use tracing::{event, span, Level, instrument};

#[derive(Debug, FromRow)]
pub struct Pairing {
	pub group_channel_id: String,
	pub round: i32,
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

#[instrument]
pub async fn db_init() -> Result<sqlx::Pool<Postgres>, sqlx::Error> {
    dotenv::dotenv().ok();

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(env::var("DATABASE_URL").unwrap().as_str())
        .await?;

    Ok(pool)
}

#[instrument]
pub async fn db_insert_list(pool: &sqlx::Pool<Postgres>, pairings: Vec<Pairing>) -> Result<(), sqlx::Error> {
    // let test_pair = Pairing { group_channel_id: "U2123".to_string(), round: 1_i32.to_string(), meeting_status: MeetingStatus::Open, names: vec!["dan".to_string(), "gurnoor".to_string()] };
    
    for pairing in pairings {
        sqlx::query("INSERT INTO ponchiks (group_channel_id, meeting_status, round, names) VALUES ($1, $2, $3, $4)")
        .bind(pairing.group_channel_id)
        .bind(pairing.meeting_status)
        .bind(pairing.round)
        .bind(pairing.names) // sus unwrap
        .execute(pool)
        .await?;

    }

    Ok(())
}

#[instrument]
pub async fn db_insert_single(pool: &sqlx::Pool<Postgres>, pairing: Pairing) -> Result<(), sqlx::Error> {
    // let test_pair = Pairing { group_channel_id: "U2123".to_string(), round: 1_i32.to_string(), meeting_status: MeetingStatus::Open, names: vec!["dan".to_string(), "gurnoor".to_string()] };

    sqlx::query("INSERT INTO ponchiks (group_channel_id, meeting_status, round, names) VALUES ($1, $2, $3, $4)")
        .bind(pairing.group_channel_id)
        .bind(pairing.meeting_status)
        .bind(pairing.round)
        .bind(pairing.names) // sus unwrap
        .execute(pool)
        .await?;

    Ok(())
}

#[instrument]
pub async fn db_read_by_groupid(pool: &sqlx::Pool<Postgres>, channel_id: String) -> Result<Pairing, sqlx::Error> {
    let select_query = 
        sqlx::query_as::<_, Pairing>("SELECT * FROM ponchiks WHERE group_channel_id = ($1)")
        .bind(channel_id);

    let pairing: Pairing = select_query.fetch_one(pool).await?;

    Ok(pairing)
}

#[instrument]
pub async fn db_find_max_round(pool: &sqlx::Pool<sqlx::Postgres>) -> Result<i32, sqlx::Error> {
    let select_query = sqlx::query(
        "SELECT MAX(round)::INTEGER FROM ponchiks"
    );

    let result = select_query.fetch_one(pool).await?;
    let val = result.get::<Option<i32>, &str>("max").unwrap_or(0);
    
    // If result is None, the db is empty and 0 should be returned.
    Ok(val)
}

#[instrument]
pub async fn db_find_all_status(pool: &sqlx::Pool<Postgres>, status: MeetingStatus) -> Result<Vec<Pairing>, sqlx::Error> {
    let select_query = 
        sqlx::query_as::<_, Pairing>("SELECT * FROM ponchiks WHERE meeting_status = ($1)")
        .bind(status);

    let pairings: Vec<Pairing> = select_query.fetch_all(pool).await?;

    Ok(pairings)
}

#[instrument]
pub async fn db_find_all_status2(pool: &sqlx::Pool<Postgres>, status1: MeetingStatus, status2: MeetingStatus) -> Result<Vec<Pairing>, sqlx::Error> {
    let select_query = 
        sqlx::query_as::<_, Pairing>("SELECT * FROM ponchiks WHERE meeting_status = ($1) OR meeting_status = ($2)")
        .bind(status1)
        .bind(status2);

    let pairings: Vec<Pairing> = select_query.fetch_all(pool).await?;

    Ok(pairings)
}

pub async fn db_find_all_round(pool: &sqlx::Pool<Postgres>, round: i16) -> Result<Vec<Pairing>, sqlx::Error> {
    let select_query = 
        sqlx::query_as::<_, Pairing>("SELECT * FROM ponchiks WHERE round = ($1)")
        .bind(round);

    let pairings: Vec<Pairing> = select_query.fetch_all(pool).await?;

    Ok(pairings)
}

// Nevermind I'm not using this function lmao there's no way it's preferable to just some code duplication
// #[instrument]
// pub async fn db_find_all_statuses(pool: &sqlx::Pool<Postgres>, statuses: Vec<MeetingStatus>) -> Result<Vec<Pairing>, sqlx::Error> {
//     if statuses.is_empty() {
//         event!(Level::DEBUG, "Empty vector of statuses inputted!");
//         return Ok(Vec::new());
//     }

//     /*
//      * Build a string which chains OR conditions of meeting statuses
//      * meeting_status = ($1) = 21 characters
//      * [ OR ] = 4 characters
//      */
//     let mut query_string = String::with_capacity((29 + (statuses.len() * 21) + ((statuses.len() - 1) * 4))); // Adjust the capacity based on your expected query size
//     query_string.push_str("SELECT * FROM ponchiks WHERE ");


//     for (index, _) in statuses.iter().enumerate() {
//         if index > 0 {
//             query_string.push_str(" OR ");
//         }
//         query_string.push_str("meeting_status = $");
//         query_string.push_str(&(index + 1).to_string());
//     }

//     let mut select_query = sqlx::query_as::<_, Pairing>(&query_string);

//     for status in statuses {
//         select_query = select_query.bind(status);
//     }

//     let pairings: Vec<Pairing> = select_query.fetch_all(pool).await?;

//     Ok(pairings)
// }


// TODO: consider checking old status before updating
#[instrument]
pub async fn db_update_status(pool: &sqlx::Pool<Postgres>, channel_id: String, new_status: MeetingStatus) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE ponchiks SET meeting_status = $1 WHERE group_channel_id = $2")
        .bind(new_status)
        .bind(channel_id)
        .execute(pool)
        .await?;

    Ok(())
}