//! Convert between Upid and Uuid.

use crate::Upid;
use uuid::Uuid;

impl From<Uuid> for Upid {
    fn from(uuid: Uuid) -> Self {
        Upid(uuid.as_u128())
    }
}

impl From<Upid> for Uuid {
    fn from(upid: Upid) -> Self {
        Uuid::from_u128(upid.0)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn uuid_cycle() {
        let want = Upid::new("user");
        let uuid: Uuid = want.into();
        let got: Upid = uuid.into();

        assert_eq!(got, want)
    }
}
