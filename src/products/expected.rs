use failure::Error;

use crate::configuration::*;
use crate::products::unexpected::*;


#[derive(Debug, Clone, Serialize, Deserialize, Fail, PartialEq)]
/// Describes all supported page expectations
pub enum PageExpectation {

    /// Valid error code
    #[fail(display = "Passed ValidCode: '{}'", _0)]
    ValidCode (u32),

    /// Valid content regex match
    #[fail(display = "Passed ValidContent: '{}'", _0)]
    ValidContent (String),

    /// Valid content length
    #[fail(display = "Passed ValidLength: '{}'", _0)]
    ValidLength (usize),

}


impl Default for PageExpectation {
    fn default() -> PageExpectation {
        PageExpectation::ValidCode(CHECK_DEFAULT_SUCCESSFUL_HTTP_CODE)
    }
}


/// Page expectations type
pub type PageExpectations = Vec<PageExpectation>;


#[derive(Debug, Copy, Clone, Serialize, Deserialize, Fail)]
/// Describes all supported domain expectations
pub enum DomainExpectation {

    /// Domain expiry minimum period in days
    #[fail(display = "Passed ValidExpiryPeriod: '{}'", _0)]
    ValidExpiryPeriod (i32),

}


impl Default for DomainExpectation {
    fn default() -> DomainExpectation {
        DomainExpectation::ValidExpiryPeriod(CHECK_SSL_DAYS_EXPIRATION)
    }
}


/// Domain expectations type
pub type DomainExpectations = Vec<DomainExpectation>;
