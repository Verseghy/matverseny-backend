use super::error;

error!(INTERNAL, INTERNAL_SERVER_ERROR, 0, "internal server error");
error!(JSON_MISSING_FIELDS, BAD_REQUEST, 1, "missing fields");
error!(JSON_SYNTAX_ERROR, BAD_REQUEST, 2, "syntax error");
error!(
    JSON_CONTENT_TYPE,
    BAD_REQUEST, 3, "missing or wrong content-type"
);
error!(JSON_VALIDATE_INVALID, BAD_REQUEST, 4, "invalid data");
error!(DUPLICATE_TEAM_NAME, BAD_REQUEST, 5, "team name exists");
error!(JOIN_CODE_NOT_FOUND, BAD_REQUEST, 6, "join code not found");
error!(LOCKED_TEAM, BAD_REQUEST, 7, "locked team");
error!(ALREADY_IN_TEAM, BAD_REQUEST, 8, "already in team");
error!(COULD_NOT_GET_CLAIMS, BAD_REQUEST, 9, "could not get claims");
error!(USER_ALREADY_EXISTS, BAD_REQUEST, 10, "user already exists");
error!(USER_NOT_IN_TEAM, BAD_REQUEST, 11, "user is not in a team");
error!(
    USER_NOT_REGISTERED,
    BAD_REQUEST, 12, "user is not registered"
);
