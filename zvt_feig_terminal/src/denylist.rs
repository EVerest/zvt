//! The blacklist of application-ids. If this list becomes too large - change
//! it for a whitelist. Possible lists can be found under
//! https://ambimat.com/developer-resources/list-of-application-identifiers-aid/
//! https://emv.cool/2020/12/23/Complete-list-of-application-identifiers-AID/
//! https://www.eftlab.com/knowledge-base/complete-list-of-application-identifiers-aid

pub const APPLICATION_ID_DENYLIST_PREFIX: &[&str] = &["a00000083"];
