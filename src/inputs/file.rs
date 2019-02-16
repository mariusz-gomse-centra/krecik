use curl::multi::{Easy2Handle, Multi};
use ssl_expiration::SslExpiration;
use curl::easy::{Easy2, Handler, WriteError};
use std::io::{Error, ErrorKind};
use std::time::Duration;

use crate::configuration::*;
use crate::utilities::*;
use crate::inputs::check::*;
use crate::checks::page::*;
use crate::checks::domain::*;
use crate::products::expected::*;
use crate::products::unexpected::*;
use crate::products::history::*;


/// NOTE: Pigeon (previous implementation) supported list of checks per file. TravMole will require each JSON to be separate file.
///       Decission is justified by lack of JSON comment ability, and other file-specific and sync troubles,
///       but also for future editing/ enable/ disable abilities that would be much more complicated with support of several checks per file.


#[derive(Debug, Clone, Serialize, Deserialize)]
/// FileCheck structure
pub struct FileCheck {

    /// Unique check name
    pub name: Option<String>,

    /// Domains to check
    pub domains: Option<Domains>,

    /// Pages to check
    pub pages: Option<Pages>,

    /// Slack Webhook
    pub alert_webhook: Option<String>,

    /// Slack alert channel
    pub alert_channel: Option<String>,

}


impl Checks<FileCheck> for FileCheck {


    fn load(name: &str) -> Result<FileCheck, Error> {
        let check_file = format!("{}/{}.json", CHECKS_DIR, &name);
        read_text_file(&check_file)
            .and_then(|file_contents| {
                serde_json::from_str(&file_contents.to_string())
                    .map_err(|err| Error::new(ErrorKind::Other, err.to_string()))
            })
    }


    fn execute(&self) -> History {
        let mut history = History::empty();

        let page_check = FileCheck::check_pages(self.pages.clone())
            .unwrap_or_else(|_| {
                warn!("No pages to check!");
                History::empty()
            });
        history = history.merge(page_check);

        let domain_check = FileCheck::check_domains(self.domains.clone())
            .unwrap_or_else(|_| {
                debug!("No domains to check!");
                History::empty()
            });
        history = history.merge(domain_check);
        history
    }


}
