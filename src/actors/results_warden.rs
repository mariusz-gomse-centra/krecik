use crate::{
    utilities::{produce_list_absolute, read_text_file},
    Notificator, Notify, Stories, STORIES_TO_KEEP_COUNT, STORIES_TO_VALIDATE_COUNT,
};
use actix::prelude::*;
use std::fs;
use tracing::{debug, info, log::trace, warn};


/// ResultsWarden actor will check stories result and send alert notification if necessary
#[derive(Debug, Copy, Clone)]
pub struct ResultsWarden;


/// Validates results history
#[derive(Message, Debug, Clone)]
#[rtype(result = "()")]
pub struct ValidateResults(pub Addr<Notificator>);


impl Handler<ValidateResults> for ResultsWarden {
    type Result = ();

    fn handle(&mut self, val: ValidateResults, _ctx: &mut Self::Context) -> Self::Result {
        debug!("ResultsWarden validates results…");
        let stories_glob = "/tmp/krecik-history-*.json";
        let files_list = produce_list_absolute(stories_glob)
            .into_iter()
            .rev()
            .take(STORIES_TO_VALIDATE_COUNT)
            .collect::<Vec<String>>();
        if files_list.is_empty() {
            info!("No results. Nothing to validate.");
            return;
        }

        debug!("Last stories file name: {}", &files_list[0]);
        let last_stories: Stories =
            serde_json::from_str(&read_text_file(&files_list[0]).unwrap_or_default())
                .unwrap_or_default();
        if last_stories.is_empty() {
            warn!("Stories seems to be incomplete? Skipping validation until next iteration.");
            return;
        }
        let last_stories_errors = last_stories
            .into_iter()
            .filter(|entry| entry.error.is_some())
            .collect::<Stories>();

        if files_list.len() < STORIES_TO_VALIDATE_COUNT {
            info!(
                "Less than {STORIES_TO_VALIDATE_COUNT} stories available, skipping validation…"
            );
        } else {
            debug!(
                "Validating last stories from {STORIES_TO_VALIDATE_COUNT} recent files: {files_list:?}"
            );

            let old_files_list = produce_list_absolute(stories_glob)
                .into_iter()
                .rev()
                .skip(STORIES_TO_KEEP_COUNT)
                .collect::<Vec<String>>();
            for old_file in &old_files_list {
                trace!("Wiping out old stories: {old_files_list:?}");
                fs::remove_file(&old_file).unwrap_or_default();
            }

            let previous_stories: Stories =
                serde_json::from_str(&read_text_file(&files_list[1]).unwrap_or_default())
                    .unwrap_or_default();
            let previous_stories_errors = previous_stories
                .into_iter()
                .filter(|entry| entry.error.is_some())
                .collect::<Stories>();

            let old_previous_stories: Stories =
                serde_json::from_str(&read_text_file(&files_list[2]).unwrap_or_default())
                    .unwrap_or_default();
            let old_previous_stories_errors = old_previous_stories
                .into_iter()
                .filter(|entry| entry.error.is_some())
                .collect::<Stories>();

            match (
                last_stories_errors.is_empty(),
                previous_stories_errors.is_empty(),
                old_previous_stories_errors.is_empty(),
            ) {
                (true, true, true) => {
                    debug!("No error Stories");
                }
                (..) => {
                    debug!("Error Stories[0]: {last_stories_errors:?}");
                    debug!("Error Stories[1]: {previous_stories_errors:?}");
                    debug!("Error Stories[2]: {old_previous_stories_errors:?}");
                }
            }

            let notifier = val.0;
            notifier.do_send(Notify(
                [
                    last_stories_errors,
                    previous_stories_errors,
                    old_previous_stories_errors,
                ]
                .concat(),
            ));
        }
    }
}


impl Actor for ResultsWarden {
    type Context = SyncContext<Self>;
}
