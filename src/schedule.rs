use chrono::{DateTime, Local, Duration, NaiveDateTime};

#[derive(Eq, PartialEq, Clone, Debug)]
pub(crate) struct Schedule {
    pub start_time: DateTime<Local>,
    pub duration: Option<Duration>,
}

impl Schedule {
    pub fn new(
        mut start_time: DateTime<Local>, 
        duration: Option<Duration>,
        skip_first_run: Option<bool>
    ) -> Self {
        if start_time < Local::now()  {
            if skip_first_run.is_some_and(|x| x == true) {
                match duration {
                    Some(dur) => {
                        start_time = start_time + dur;
                    }
                    None => {start_time = Local::now()}
                }
                
            } else {
                start_time = Local::now()
            }
        } 

        Schedule {
            start_time: start_time,
            duration: duration,
        }
    } 
}