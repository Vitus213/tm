pub fn format_duration_minutes_style(seconds: i64) -> String {
    let minutes = seconds / 60;
    if minutes < 60 {
        return format!("{minutes} min");
    }

    let hours = minutes / 60;
    let remaining_minutes = minutes % 60;
    format!("{hours}h {remaining_minutes}m")
}

#[cfg(test)]
mod tests {
    use super::format_duration_minutes_style;

    #[test]
    fn formats_sub_minute_durations_as_zero_minutes() {
        assert_eq!(format_duration_minutes_style(0), "0 min");
        assert_eq!(format_duration_minutes_style(45), "0 min");
        assert_eq!(format_duration_minutes_style(59), "0 min");
    }

    #[test]
    fn formats_minute_range_durations_as_whole_minutes() {
        assert_eq!(format_duration_minutes_style(60), "1 min");
        assert_eq!(format_duration_minutes_style(65), "1 min");
        assert_eq!(format_duration_minutes_style(3599), "59 min");
        assert_eq!(format_duration_minutes_style(59 * 60), "59 min");
    }

    #[test]
    fn formats_hour_plus_durations_as_hours_and_minutes() {
        assert_eq!(format_duration_minutes_style(60 * 60), "1h 0m");
        assert_eq!(format_duration_minutes_style(61 * 60), "1h 1m");
        assert_eq!(format_duration_minutes_style((2 * 60 + 5) * 60), "2h 5m");
    }
}
