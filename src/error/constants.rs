use super::error;

// id 0: internal error
error!(JSON_MISSING_FIELDS, BAD_REQUEST, 1, "missing fields");
error!(JSON_SYNTAX_ERROR, BAD_REQUEST, 2, "syntax error");
error!(
    JSON_CONTENT_TYPE,
    BAD_REQUEST, 3, "missing or wrong content-type"
);
error!(JSON_VALIDATE_INVALID, BAD_REQUEST, 4, "invalid data");
