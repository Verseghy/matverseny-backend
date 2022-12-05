use super::const_error;

const_error! {
    #[error(0, "internal server error")]
    #[status(INTERNAL_SERVER_ERROR)]
    const INTERNAL;
}
const_error! {
    #[error(1, "missing fields")]
    #[status(UNPROCESSABLE_ENTITY)]
    const JSON_MISSING_FIELDS;
}
const_error! {
    #[error(2, "syntax error")]
    #[status(BAD_REQUEST)]
    const JSON_SYNTAX_ERROR;
}
const_error! {
    #[error(3, "missing or wrong content-type")]
    #[status(BAD_REQUEST)]
    const JSON_CONTENT_TYPE;
}
const_error! {
    #[error(4, "invalid data")]
    #[status(BAD_REQUEST)]
    const JSON_VALIDATE_INVALID;
}
const_error! {
    #[error(5, "team name exists")]
    #[status(BAD_REQUEST)]
    const DUPLICATE_TEAM_NAME;
}
const_error! {
    #[error(6, "join code not found")]
    #[status(BAD_REQUEST)]
    const JOIN_CODE_NOT_FOUND;
}
const_error! {
    #[error(7, "locked team")]
    #[status(BAD_REQUEST)]
    const LOCKED_TEAM;
}
const_error! {
    #[error(8, "already in team")]
    #[status(BAD_REQUEST)]
    const ALREADY_IN_TEAM;
}
const_error! {
    #[error(9, "could not get claims")]
    #[status(UNAUTHORIZED)]
    const COULD_NOT_GET_CLAIMS;
}
const_error! {
    #[error(10, "user already exists")]
    #[status(BAD_REQUEST)]
    const USER_ALREADY_EXISTS;
}
const_error! {
    #[error(11, "user is not in a team")]
    #[status(BAD_REQUEST)]
    const USER_NOT_IN_TEAM;
}
const_error! {
    #[error(12, "user is not registered")]
    #[status(FORBIDDEN)]
    const USER_NOT_REGISTERED;
}
const_error! {
    #[error(13, "user must be the owner of the team")]
    #[status(FORBIDDEN)]
    const USER_NOT_OWNER;
}
const_error! {
    #[error(14, "user is not a member of the team")]
    #[status(BAD_REQUEST)]
    const NO_SUCH_MEMBER;
}
const_error! {
    #[error(15, "user must be the owner or the co-owner of the team")]
    #[status(FORBIDDEN)]
    const USER_NOT_COOWNER;
}
const_error! {
    #[error(16, "cannot kick the owner of a team")]
    #[status(FORBIDDEN)]
    const CANNOT_KICK_OWNER;
}
const_error! {
    #[error(17, "cannot kick yourself")]
    #[status(FORBIDDEN)]
    const CANNOT_KICK_THEMSELF;
}
const_error! {
    #[error(18, "failed to generate join code")]
    #[status(INTERNAL_SERVER_ERROR)]
    const FAILED_TO_GENERATE_JOIN_CODE;
}
const_error! {
    #[error(19, "the owner cannot leave the team")]
    #[status(FORBIDDEN)]
    const OWNER_CANNOT_LEAVE;
}
const_error! {
    #[error(20, "invalid jwt token")]
    #[status(BAD_REQUEST)]
    const JWT_INVALID_TOKEN;
}
