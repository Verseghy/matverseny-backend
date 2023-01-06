use super::const_error;

const_error! {
    #[error("M000", "internal server error")]
    #[status(INTERNAL_SERVER_ERROR)]
    const INTERNAL;
}
const_error! {
    #[error("M001", "missing fields")]
    #[status(UNPROCESSABLE_ENTITY)]
    const JSON_MISSING_FIELDS;
}
const_error! {
    #[error("M002", "syntax error")]
    #[status(BAD_REQUEST)]
    const JSON_SYNTAX_ERROR;
}
const_error! {
    #[error("M003", "missing or wrong content-type")]
    #[status(BAD_REQUEST)]
    const JSON_CONTENT_TYPE;
}
const_error! {
    #[error("M004", "invalid data")]
    #[status(BAD_REQUEST)]
    const JSON_VALIDATE_INVALID;
}
const_error! {
    #[error("M005", "team name exists")]
    #[status(BAD_REQUEST)]
    const DUPLICATE_TEAM_NAME;
}
const_error! {
    #[error("M006", "join code not found")]
    #[status(BAD_REQUEST)]
    const JOIN_CODE_NOT_FOUND;
}
const_error! {
    #[error("M007", "locked team")]
    #[status(BAD_REQUEST)]
    const LOCKED_TEAM;
}
const_error! {
    #[error("M008", "already in team")]
    #[status(BAD_REQUEST)]
    const ALREADY_IN_TEAM;
}
const_error! {
    #[error("M009", "could not get claims")]
    #[status(UNAUTHORIZED)]
    const COULD_NOT_GET_CLAIMS;
}
const_error! {
    #[error("M010", "user already exists")]
    #[status(BAD_REQUEST)]
    const USER_ALREADY_EXISTS;
}
const_error! {
    #[error("M011", "user is not in a team")]
    #[status(BAD_REQUEST)]
    const USER_NOT_IN_TEAM;
}
const_error! {
    #[error("M012", "user is not registered")]
    #[status(FORBIDDEN)]
    const USER_NOT_REGISTERED;
}
const_error! {
    #[error("M013", "user must be the owner of the team")]
    #[status(FORBIDDEN)]
    const USER_NOT_OWNER;
}
const_error! {
    #[error("M014", "user is not a member of the team")]
    #[status(BAD_REQUEST)]
    const NO_SUCH_MEMBER;
}
const_error! {
    #[error("M015", "user must be the owner or the co-owner of the team")]
    #[status(FORBIDDEN)]
    const USER_NOT_COOWNER;
}
const_error! {
    #[error("M016", "cannot kick the owner of a team")]
    #[status(FORBIDDEN)]
    const CANNOT_KICK_OWNER;
}
const_error! {
    #[error("M017", "cannot kick yourself")]
    #[status(FORBIDDEN)]
    const CANNOT_KICK_THEMSELF;
}
const_error! {
    #[error("M018", "failed to generate join code")]
    #[status(INTERNAL_SERVER_ERROR)]
    const FAILED_TO_GENERATE_JOIN_CODE;
}
const_error! {
    #[error("M019", "the owner cannot leave the team")]
    #[status(FORBIDDEN)]
    const OWNER_CANNOT_LEAVE;
}
const_error! {
    #[error("M020", "invalid jwt token")]
    #[status(BAD_REQUEST)]
    const JWT_INVALID_TOKEN;
}
const_error! {
    #[error("M021", "the message has the wrong type")]
    const WEBSOCKET_WRONG_MESSAGE_TYPE;
}
const_error! {
    #[error("M022", "did not receive auth token in time")]
    const WEBSOCKET_AUTH_TIMEOUT;
}
const_error! {
    #[error("M023", "no such problem")]
    #[status(NOT_FOUND)]
    const PROBLEM_NOT_FOUND;
}
const_error! {
    #[error("M024", "database error")]
    #[status(INTERNAL_SERVER_ERROR)]
    const DATABASE_ERROR;
}
const_error! {
    #[error("M025", "failed to deserialize json")]
    #[status(INTERNAL_SERVER_ERROR)]
    const JSON_DESERIALIZE;
}
const_error! {
    #[error("M026", "kafka error")]
    #[status(INTERNAL_SERVER_ERROR)]
    const KAFKA_ERROR;
}
const_error! {
    #[error("M027", "websocket error")]
    const WEBSOCKET_ERROR;
}
