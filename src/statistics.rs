use slick_models::lh_models::Report;

pub fn calculate_mean(reports: &Vec<Report>) -> Option<f64> {
    let mut sum = 0.0 as f64;

    for report in reports {
        sum += report.categories().performance().score();
    }

    let count = reports.len();

    match count {
        positive if positive > 0 => Some(sum / count as f64),
        _ => None,
    }
}

pub fn calculate_std_deviation(reports: &Vec<Report>) -> Option<f64> {
    match (calculate_mean(reports), reports.len()) {
        (Some(data_mean), count) if count > 0 => {
            let mut sum_diff = 0 as f64;
            for report in reports {
                let diff = data_mean - (*report.categories().performance().score() as f64);

                sum_diff += diff * diff;
            }

            let variance = sum_diff / count as f64;

            Some(variance.sqrt())
        },
        _ => None
    }
}
