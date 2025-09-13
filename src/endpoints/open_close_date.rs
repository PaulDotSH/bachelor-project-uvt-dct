use anyhow::anyhow;
use axum::extract::State;
use axum::response::{Html, Redirect};
use axum::Form;
use time::{OffsetDateTime, Duration, macros::format_description};
use sailfish::TemplateSimple;
use serde::{Deserialize, Serialize};
use crate::{constants::GMT, error::AppError, lib::AppState};

#[derive(sailfish_minify::TemplateSimple)]
#[template(path = "open_close_date.stpl")]
struct OpenCloseDateTemplate {
    date_data: Option<StartEndDateTz>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StartEndDate {
    pub start_date: OffsetDateTime,
    pub end_date: OffsetDateTime,
}

struct StartEndDateTz {
    start_date: OffsetDateTime,
    end_date: OffsetDateTime,
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
            // Convert to UTC+3 (Bucharest)
            let tz_offset = time::UtcOffset::from_hms(3, 0, 0).unwrap();
            let start_date_bucharest = r.start_date.to_offset(tz_offset);
            let end_date_bucharest = r.end_date.to_offset(tz_offset);
            
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
    let tz_offset = time::UtcOffset::from_hms(3, 0, 0).unwrap();

    let format = format_description!("[year]-[month]-[day]T[hour]:[minute]");
    let start_date_local = time::PrimitiveDateTime::parse(&form.start_date, format)?;
    let end_date_local = time::PrimitiveDateTime::parse(&form.end_date, format)?;

    let duration = end_date_local - start_date_local;
    if duration < Duration::hours(3) {
        return Err(AppError(anyhow!("There must be at least 3 hours between the start and end date, and the end date must be later than the start date.")));
    }

    let start_date_utc = start_date_local.assume_offset(tz_offset);
    let end_date_utc = end_date_local.assume_offset(tz_offset);

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
