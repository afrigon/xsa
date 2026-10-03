#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CivilDate {
    pub year: i64,
    pub month: i64,
    pub day: i64,
}

impl CivilDate {
    // Howard Hinnant's civil_from_days, the inverse of days_since_unix_epoch.
    pub fn from_days_since_unix_epoch(days_since_unix_epoch: i64) -> CivilDate {
        let days = days_since_unix_epoch + 719_468;
        let era = days.div_euclid(146_097);
        let day_of_era = days - era * 146_097;
        let year_of_era = (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
        let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
        let month_index = (5 * day_of_year + 2) / 153;
        let day = day_of_year - (153 * month_index + 2) / 5 + 1;
        let month = if month_index < 10 {
            month_index + 3
        } else {
            month_index - 9
        };
        let year = year_of_era + era * 400 + if month <= 2 { 1 } else { 0 };

        CivilDate { year, month, day }
    }

    // Howard Hinnant's days_from_civil: days since 1970-01-01 in the proleptic Gregorian calendar.
    pub fn days_since_unix_epoch(&self) -> i64 {
        let year = if self.month <= 2 { self.year - 1 } else { self.year };
        let era = year.div_euclid(400);
        let year_of_era = year - era * 400;
        let shifted_month = if self.month > 2 { self.month - 3 } else { self.month + 9 };
        let day_of_year = (153 * shifted_month + 2) / 5 + self.day - 1;
        let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;

        era * 146_097 + day_of_era - 719_468
    }
}
