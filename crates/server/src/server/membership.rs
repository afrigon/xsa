use xsa_proto::user::User;

pub(super) enum Membership {
    User { user: User },
    Server,
}
