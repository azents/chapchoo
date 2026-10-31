//! Protocol- and persistence-independent account values.
//!
//! Credential contents cannot be accessed without the explicit boundary method:
//!
//! ```compile_fail
//! let password = chapchoo_domain::Password::parse("Aa1!xxxx".to_owned()).unwrap();
//! let plaintext = password.0;
//! ```
//!
//! Neither persistence nor protocol adapters are dependencies of this crate:
//!
//! ```compile_fail
//! use diesel::Connection;
//! use tonic::Request;
//! ```

mod display;
mod email;
mod password;
mod role;

pub use display::{DisplayPatch, DisplaySettings, InvalidDisplaySettings};
pub use email::{EmailAddress, InvalidEmail};
pub use password::{Password, PasswordPolicyError};
pub use role::{MembershipRole, TeamKind, TeamRole};
