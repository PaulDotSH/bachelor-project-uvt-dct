use anyhow::anyhow;
use axum::{
    extract::State,
    response::{Html, Redirect},
    Form,
};
use chrono::{DateTime, Duration, FixedOffset, NaiveDateTime, TimeZone, Utc};
use chrono_tz::{Europe::Bucharest, Tz};
use sailfish::TemplateOnce;
use serde::{Deserialize, Serialize};

use crate::{error::AppError, AppState, GLOBAL_TIMEZONE};

#[derive(sailfish_minify::TemplateOnce)]
#[templ(path = "open_close_date.stpl")]
struct OpenCloseDateTemplate {
    date_data: Option<StartEndDateTz>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StartEndDate {
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
}

struct StartEndDateTz {
    start_date: DateTime<Tz>,
    end_date: DateTime<Tz>,
}

pub async fn get_page(State(state): State<AppState>) -> Result<Html<String>, AppError> {
    let record = sqlx::query_as!(
        StartEndDate,
        "SELECT start_date, end_date FROM choices_open_date LIMIT 1"
    )
    .fetch_optional(&state.postgres)
    .await?;

    let ctx = match record {
        Some(r) => {
            let start_date_bucharest = r.start_date.with_timezone(&Bucharest);
            let end_date_bucharest = r.end_date.with_timezone(&Bucharest);
            OpenCloseDateTemplate {
                date_data: Some(StartEndDateTz {
                    start_date: start_date_bucharest,
                    end_date: end_date_bucharest,
                }),
            }
        }
        None => OpenCloseDateTemplate { date_data: None },
    };

    let body = ctx.render_once().map_err(|e| AppError(e.into()))?;
    Ok(Html::from(body))
}

#[derive(Deserialize)]
pub struct UpdateDateForm {
    start_date: String,
    end_date: String,
}

pub async fn update(
    State(state): State<AppState>,
    Form(form): Form<UpdateDateForm>,
) -> Result<Redirect, AppError> {
    // GMT+3
    let fixed_offset = FixedOffset::east_opt(GLOBAL_TIMEZONE).expect("Invalid timezone");

    let start_date_naive = NaiveDateTime::parse_from_str(&form.start_date, "%Y-%m-%dT%H:%M")?;
    let end_date_naive = NaiveDateTime::parse_from_str(&form.end_date, "%Y-%m-%dT%H:%M")?;

    let duration = end_date_naive - start_date_naive;
    if duration < Duration::hours(3) {
        return Err(AppError(anyhow!("There must be at least 3 hours between the start and end date, and the end date must be later than the start date.")));
    }

    let start_date_fixed = fixed_offset
        .from_local_datetime(&start_date_naive)
        .single()
        .unwrap();
    let end_date_fixed = fixed_offset
        .from_local_datetime(&end_date_naive)
        .single()
        .unwrap();

    let start_date_utc = start_date_fixed.with_timezone(&Utc);
    let end_date_utc = end_date_fixed.with_timezone(&Utc);

    sqlx::query!(
        "
        INSERT INTO choices_open_date (start_date, end_date) 
        VALUES ($1, $2)
        ON CONFLICT (id)
        DO UPDATE SET
            start_date = EXCLUDED.start_date,
            end_date = EXCLUDED.end_date
        ",
        start_date_utc,
        end_date_utc
    )
    .execute(&state.postgres)
    .await?;

    Ok(Redirect::to("/open_close_dates"))
}
