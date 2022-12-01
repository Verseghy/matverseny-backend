use super::const_error;

const_error!(INTERNAL, INTERNAL_SERVER_ERROR, 0, "internal server error");
const_error!(
    JSON_MISSING_FIELDS,
    UNPROCESSABLE_ENTITY,
    1,
    "missing fields"
);
const_error!(JSON_SYNTAX_ERROR, BAD_REQUEST, 2, "syntax error");
const_error!(
    JSON_CONTENT_TYPE,
    BAD_REQUEST,
    3,
    "missing or wrong content-type"
);
const_error!(JSON_VALIDATE_INVALID, BAD_REQUEST, 4, "invalid data");
const_error!(DUPLICATE_TEAM_NAME, BAD_REQUEST, 5, "team name exists");
const_error!(JOIN_CODE_NOT_FOUND, BAD_REQUEST, 6, "join code not found");
const_error!(LOCKED_TEAM, BAD_REQUEST, 7, "locked team");
const_error!(ALREADY_IN_TEAM, BAD_REQUEST, 8, "already in team");
const_error!(
    COULD_NOT_GET_CLAIMS,
    UNAUTHORIZED,
    9,
    "could not get claims"
);
const_error!(USER_ALREADY_EXISTS, BAD_REQUEST, 10, "user already exists");
const_error!(USER_NOT_IN_TEAM, BAD_REQUEST, 11, "user is not in a team");
const_error!(USER_NOT_REGISTERED, FORBIDDEN, 12, "user is not registered");
const_error!(
    USER_NOT_OWNER,
    FORBIDDEN,
    13,
    "user must be the owner of the team"
);
const_error!(
    NO_SUCH_MEMBER,
    BAD_REQUEST,
    14,
    "user is not a member of the team"
);
const_error!(
    USER_NOT_COOWNER,
    FORBIDDEN,
    15,
    "user must be the owner or the coowner of the team"
);
const_error!(
    CANNOT_KICK_OWNER,
    FORBIDDEN,
    16,
    "cannot kick the owner of a team"
);
const_error!(CANNOT_KICK_THEMSELF, FORBIDDEN, 17, "cannot kick yourself");
const_error!(
    FAILED_TO_GENERATE_JOIN_CODE,
    INTERNAL_SERVER_ERROR,
    18,
    "failed to generate join code"
);
const_error!(
    OWNER_CANNOT_LEAVE,
    FORBIDDEN,
    19,
    "the owner cannot leave the team"
);
