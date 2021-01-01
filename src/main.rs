//! "Traversing Mole" server

//! Crate docs

#![forbid(unsafe_code)]
#![deny(
    missing_docs,
    unstable_features,
    unsafe_code,
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unused_import_braces,
    unused_qualifications
)]
// For development:
#![allow(dead_code, unused_imports, unused_variables, deprecated)]

use krecik::{
    actors::{
        curl_multi_checker::{Checks, CurlMultiChecker},
        curl_multi_checker_pongo::Checks as ChecksPongo,
        curl_multi_checker_pongo::CurlMultiCheckerPongo,
        domain_expiry_checker::Checks as DomainChecks,
        domain_expiry_checker::DomainExpiryChecker,
    },
    api::*,
    checks::{domain::Domains, page::Method, pongo::get_pongo_hosts},
    configuration::{CHECKS_DIR, CHECK_DEFAULT_SUCCESSFUL_HTTP_CODE},
    products::{
        expected::{Expected, PageExpectation, PageExpectations},
        unexpected::{Unexpected, UnexpectedMinor},
    },
};
use krecik::{checks::generic::*, configuration::CHECK_TIMEOUT};
use krecik::{
    checks::{check::*, page::Page},
    configuration::DEFAULT_SLACK_NAME,
};
use krecik::{configuration::CHECK_CONNECTION_TIMEOUT, products::story::*};
use krecik::{
    configuration::{CHECK_MAX_CONNECTIONS, CHECK_MAX_REDIRECTIONS},
    *,
};

// use actix_derive::*;
use actix::prelude::*;
use actix_macros::main as actix_main;

#[macro_use]
extern crate fern;

use chrono::*;
use colored::Colorize;
use fern::InitError;
use krecik::api::*;
use log::*;
use std::fs;
use std::{env, env::var, path::Path};


use curl::{
    easy::{Easy2, List},
    multi::{Easy2Handle, Multi},
    Error as CurlError, MultiError,
};
use rayon::prelude::*;
use ssl_expiration2::SslExpiration;
use std::{
    io::{Error, ErrorKind},
    time::Duration,
};


fn setup_logger(level: LevelFilter) -> Result<(), InitError> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(level)
        .chain(std::io::stdout())
        .apply()?;
    Ok(())
}


#[actix_main]
async fn main() {
    let logger_level = match var("DEBUG") {
        Ok(value) => LevelFilter::Debug,
        Err(_) => LevelFilter::Info,
    };
    setup_logger(logger_level).unwrap_or_default();

    // Define system actors
    let curl_multi_checker = SyncArbiter::start(4, || CurlMultiChecker);
    let curl_multi_checker_pongo = SyncArbiter::start(4, || CurlMultiCheckerPongo);
    let domain_expiry_checker = SyncArbiter::start(4, || DomainExpiryChecker);

    // let results_warden = ResultsWarden::start(1, || )
    // let pongo_curl_actor = SyncArbiter::start(4, || CurlMultiChecker);

    // TODO: split remotes, load mappers manually and ONCE

    let pongo_checks = curl_multi_checker_pongo
        .send(ChecksPongo(all_checks_pongo_remote_pages()))
        .await;

    let domain_checks = domain_expiry_checker
        .send(DomainChecks(all_checks_pongo_remote_domains()))
        .await;

    let regular_checks = curl_multi_checker
        .send(Checks(all_checks_but_remotes()))
        .await;

    let stories = [
        domain_checks.unwrap().unwrap_or_default(),
        regular_checks.unwrap().unwrap_or_default(),
        pongo_checks.unwrap().unwrap_or_default(),
    ]
    .concat();
    // debug!(
    //     "Stories JSON: [{}]",
    //     stories
    //         .iter()
    //         .map(|story| story.to_string())
    //         .collect::<String>()
    // );
    utilities::write_append(
        "/tmp/out.json",
        &stories
            .iter()
            .map(|story| story.to_string())
            .collect::<String>(),
    );
    info!("Result stories count: {}", stories.len());

    System::current().stop();
}
