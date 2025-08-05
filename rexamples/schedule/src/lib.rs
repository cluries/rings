use rings::tokio_cron_scheduler::Job;

pub fn scone() -> Job {
    Job::new_async("1/7 * * * * *", |uuid, mut locker| {
        // println!("uuid: {}", uuid);
        Box::pin(async move {
            // println!("I run async every 7 seconds");
            let next_tick = locker.next_tick_for_job(uuid).await;
            match next_tick {
                Ok(Some(_ts)) => {
                    //println!("Next time for 7s job is {:?}", ts);

                }
                _ => {
                    println!("Could not get next tick for 7s job")
                }
            }
        })
    }).unwrap()
}
